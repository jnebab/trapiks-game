use std::cmp::Reverse;

use crate::consts::{DECISION_MARGIN, IDM_COMFORT_DECEL, IDM_MAX_ACCEL};
use crate::map::Control;
use crate::network::{Junction, LinkId, Movement, Network, SignalState};

use super::leader::current_ahead;
use super::{Occupancy, Place, VehicleStore};

pub type PriorityKey = (Reverse<(u8, u8, u8)>, u64, u32);

pub struct RuleContext<'a> {
    pub network: &'a Network,
    pub vehicles: &'a VehicleStore,
    pub occupancy: &'a Occupancy,
    pub tick: u64,
}

#[derive(Clone, Copy)]
pub struct Approach<'a> {
    pub slot: u32,
    pub link: LinkId,
    pub lane: u8,
    pub node: u32,
    pub index: u16,
    pub junction: &'a Junction,
    pub movement: &'a Movement,
    pub s: f64,
    pub v: f64,
    pub span_end: f64,
}

pub fn approach<'a>(ctx: &RuleContext<'a>, slot: u32) -> Option<Approach<'a>> {
    let index = slot as usize;
    let Place::Link { link, lane } = ctx.vehicles.place[index] else {
        return None;
    };
    let ahead = current_ahead(ctx.network, ctx.vehicles, slot);
    let (node, movement_index) = ahead.movement()?;
    let junction = ctx.network.junction(node)?;
    Some(Approach {
        slot,
        link,
        lane,
        node,
        index: movement_index,
        junction,
        movement: junction.movements.get(usize::from(movement_index))?,
        s: ctx.vehicles.s[index],
        v: ctx.vehicles.v[index],
        span_end: ahead.end()?,
    })
}

pub fn stopping_distance(v: f64) -> f64 {
    v * v / (2.0 * IDM_COMFORT_DECEL)
}

pub fn in_zone(network: &Network, vehicles: &VehicleStore, slot: u32) -> bool {
    let index = slot as usize;
    let Place::Link { link, .. } = vehicles.place[index] else {
        return false;
    };
    if vehicles.route_link(slot, 1).is_none() {
        return false;
    }
    let remaining = network.link_span(link).1 - vehicles.s[index];
    within_zone(remaining, vehicles.v[index])
}

pub fn within_zone(remaining: f64, v: f64) -> bool {
    remaining <= stopping_distance(v) + DECISION_MARGIN
}

pub fn heads_to(vehicles: &VehicleStore, slot: u32, target: LinkId) -> bool {
    vehicles.route_link(slot, 1) == Some(target)
}

pub fn time_to(d: f64, v: f64, v0: f64) -> f64 {
    let accelerating = (-v + libm::sqrt(v * v + 2.0 * IDM_MAX_ACCEL * d)) / IDM_MAX_ACCEL;
    accelerating.max(d / v0)
}

pub fn is_minor(network: &Network, approach: &Approach) -> bool {
    let control = network.nodes.control[approach.node as usize];
    if !matches!(control, Control::Stop | Control::Yield) {
        return false;
    }
    let highest = approach.junction.movements.iter().map(|m| m.rank.0).max();
    highest.is_some_and(|top| approach.movement.rank.0 < top)
}

fn signal_priority(ctx: &RuleContext, link: LinkId) -> u8 {
    let state = ctx.network.signal_state(link, ctx.tick);
    let lit = matches!(state, Some(SignalState::Green | SignalState::Amber));
    u8::from(lit || ctx.network.signals().is_internal(link))
}

fn effective_rank(ctx: &RuleContext, node: u32, movement: &Movement) -> (u8, u8, u8) {
    if ctx.network.nodes.control[node as usize] == Control::AllWayStop {
        return (0, 0, 0);
    }
    let (class, turn) = movement.rank;
    (signal_priority(ctx, movement.from_link), class, turn)
}

pub fn movement_priority(
    ctx: &RuleContext,
    slot: u32,
    node: u32,
    movement: &Movement,
) -> PriorityKey {
    let index = slot as usize;
    (
        Reverse(effective_rank(ctx, node, movement)),
        ctx.vehicles.arrival_tick[index],
        ctx.vehicles.id[index],
    )
}

pub fn priority_key(ctx: &RuleContext, approach: &Approach) -> PriorityKey {
    movement_priority(ctx, approach.slot, approach.node, approach.movement)
}

pub fn beats(ctx: &RuleContext, a: &Approach, b: &Approach) -> bool {
    priority_key(ctx, a) < priority_key(ctx, b)
}
