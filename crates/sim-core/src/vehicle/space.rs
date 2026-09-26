use crate::consts::IDM_MIN_GAP;
use crate::network::{LinkId, Network, road_of};

use super::approach::{Approach, RuleContext, heads_to};
use super::leader::current_ahead;
use super::occupancy::{link_key, movement_key};
use super::{Occupancy, VehicleStore};

pub struct Free {
    pub length: f64,
    pub empty: bool,
}

struct Demand {
    required: f64,
    counted: usize,
}

impl Demand {
    fn add(&mut self, length: f64) {
        self.required += length + IDM_MIN_GAP;
        self.counted += 1;
    }

    fn fits(&self, free: &Free) -> bool {
        free.length >= self.required || (free.empty && self.counted == 0)
    }
}

pub fn space_ok(ctx: &RuleContext, approach: &Approach) -> bool {
    predecessor_committed(ctx, approach) && room_ok(ctx, approach)
}

pub fn room_ok(ctx: &RuleContext, approach: &Approach) -> bool {
    target_has_room(ctx, approach) && cluster_exit_has_room(ctx, approach)
}

pub fn predecessor_committed(ctx: &RuleContext, approach: &Approach) -> bool {
    let key = link_key(approach.link, approach.lane);
    ctx.occupancy
        .ahead(approach.slot, key)
        .last()
        .is_none_or(|entry| ctx.vehicles.committed[entry.slot as usize])
}

fn target_has_room(ctx: &RuleContext, approach: &Approach) -> bool {
    let ahead = current_ahead(ctx.network, ctx.vehicles, approach.slot);
    let Some(lane) = ahead.landing() else {
        return false;
    };
    let target = approach.movement.to_link;
    let free = free_space(ctx.network, ctx.vehicles, ctx.occupancy, target, lane);
    let own = ctx.vehicles.length(approach.slot) + IDM_MIN_GAP;
    let mut demand = Demand {
        required: own * f64::from(1 + u8::from(is_ring_entry(ctx.network, approach))),
        counted: 0,
    };
    let occupants = ctx
        .occupancy
        .slots_on(movement_key(approach.node, approach.index));
    for slot in occupants {
        demand.add(ctx.vehicles.length(slot));
    }
    if !demand.fits(&free) {
        return false;
    }
    for entry in ctx.occupancy.link_block(approach.link) {
        let slot = entry.slot;
        if slot == approach.slot || !ctx.vehicles.committed[slot as usize] {
            continue;
        }
        if heads_to(ctx.vehicles, slot, target) {
            demand.add(ctx.vehicles.length(slot));
            if !demand.fits(&free) {
                return false;
            }
        }
    }
    true
}

fn is_ring_entry(network: &Network, approach: &Approach) -> bool {
    let movement = approach.movement;
    !network.roads.is_roundabout(road_of(movement.from_link))
        && network.roads.is_roundabout(road_of(movement.to_link))
}

pub fn free_space(
    network: &Network,
    vehicles: &VehicleStore,
    occupancy: &Occupancy,
    link: LinkId,
    lane: u8,
) -> Free {
    let (start, end) = network.link_span(link);
    match occupancy.last_on(link_key(link, lane)) {
        Some((slot, s)) => Free {
            length: s - vehicles.length(slot) - start,
            empty: false,
        },
        None => Free {
            length: end - start,
            empty: true,
        },
    }
}

fn cluster_exit_has_room(ctx: &RuleContext, approach: &Approach) -> bool {
    let signals = ctx.network.signals();
    if signals.cluster_of(approach.node).is_none() || signals.is_internal(approach.link) {
        return true;
    }
    let route = ctx.vehicles.route(approach.slot);
    let rest = &route[ctx.vehicles.cursor(approach.slot) + 1..];
    let Some(&exit) = rest.iter().find(|&&link| !signals.is_internal(link)) else {
        return true;
    };
    let needed = ctx.vehicles.length(approach.slot) + IDM_MIN_GAP;
    (0..ctx.network.link_lanes(exit)).any(|lane| {
        let free = free_space(ctx.network, ctx.vehicles, ctx.occupancy, exit, lane);
        free.empty || free.length >= needed
    })
}
