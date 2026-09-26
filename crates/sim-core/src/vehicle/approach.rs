use std::cmp::Reverse;

use crate::consts::{DECISION_MARGIN, IDM_COMFORT_DECEL, IDM_MAX_ACCEL};
use crate::map::Control;
use crate::network::{Junction, LinkId, Movement, Network, SignalState};

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
    let next = ctx.vehicles.route_link(slot, 1)?;
    let node = ctx.network.link_to(link);
    let junction = ctx.network.junction(node)?;
    let movement_index = junction.movement_index(link, next)?;
    Some(Approach {
        slot,
        link,
        lane,
        node,
        index: u16::try_from(movement_index).ok()?,
        junction,
        movement: &junction.movements[movement_index],
        s: ctx.vehicles.s[index],
        v: ctx.vehicles.v[index],
        span_end: ctx.network.link_span(link).1,
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
    remaining <= stopping_distance(vehicles.v[index]) + DECISION_MARGIN
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

fn effective_rank(ctx: &RuleContext, approach: &Approach) -> (u8, u8, u8) {
    if ctx.network.nodes.control[approach.node as usize] == Control::AllWayStop {
        return (0, 0, 0);
    }
    let (class, turn) = approach.movement.rank;
    (signal_priority(ctx, approach.link), class, turn)
}

pub fn priority_key(ctx: &RuleContext, approach: &Approach) -> PriorityKey {
    let index = approach.slot as usize;
    (
        Reverse(effective_rank(ctx, approach)),
        ctx.vehicles.arrival_tick[index],
        ctx.vehicles.id[index],
    )
}

pub fn beats(ctx: &RuleContext, a: &Approach, b: &Approach) -> bool {
    priority_key(ctx, a) < priority_key(ctx, b)
}
