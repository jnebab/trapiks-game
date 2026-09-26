use crate::consts::{CAR_LENGTH, IDM_MIN_GAP};
use crate::network::LinkId;

use super::approach::{Approach, RuleContext, heads_to};
use super::lanes::landing_lane;
use super::occupancy::{link_key, movement_key};

const SLOT_LENGTH: f64 = CAR_LENGTH + IDM_MIN_GAP;

struct Free {
    length: f64,
    empty: bool,
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
    let route = ctx.vehicles.route(approach.slot);
    let cursor = ctx.vehicles.cursor(approach.slot);
    let Some(lane) = landing_lane(ctx.network, approach.movement, route, cursor + 1) else {
        return false;
    };
    let target = approach.movement.to_link;
    let free = free_space(ctx, target, lane);
    let mut n = ctx
        .occupancy
        .count(movement_key(approach.node, approach.index));
    let key = link_key(approach.link, approach.lane);
    for entry in ctx.occupancy.ahead(approach.slot, key).iter().rev() {
        if !fits(n, &free) {
            return false;
        }
        let slot = entry.slot;
        if ctx.vehicles.committed[slot as usize] && heads_to(ctx.vehicles, slot, target) {
            n += 1;
        }
    }
    fits(n, &free)
}

fn fits(n: usize, free: &Free) -> bool {
    let required = SLOT_LENGTH * (1 + n) as f64;
    free.length >= required || (free.empty && n == 0)
}

fn free_space(ctx: &RuleContext, link: LinkId, lane: u8) -> Free {
    let (start, end) = ctx.network.link_span(link);
    match ctx.occupancy.last_on(link_key(link, lane)) {
        Some((_, s)) => Free {
            length: s - start,
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
    (0..ctx.network.link_lanes(exit)).any(|lane| {
        let free = free_space(ctx, exit, lane);
        free.empty || free.length >= SLOT_LENGTH
    })
}
