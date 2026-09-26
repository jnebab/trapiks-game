use crate::consts::{
    APPROACH_SCAN, CONFLICT_CLEAR_MARGIN, MINOR_GAP, PRIORITY_GAP, WAIT_TIMEOUT_TICKS,
};
use crate::map::Control;
use crate::network::{Conflict, Movement, SignalState};

use super::approach::{
    Approach, RuleContext, approach, approach_time, beats, excluded, heads_to, in_zone, is_minor,
    stopping_distance,
};
use super::occupancy::{link_key, movement_key};
use super::space::{predecessor_committed, room_ok, space_ok};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Blocked,
    Yielding,
}

pub fn may_enter(ctx: &RuleContext, slot: u32) -> bool {
    evaluate(ctx, slot) == Verdict::Pass
}

pub fn evaluate(ctx: &RuleContext, slot: u32) -> Verdict {
    let Some(a) = approach(ctx, slot) else {
        return Verdict::Blocked;
    };
    if !passes_hard_rules(ctx, &a) || !clear(ctx, &a) {
        return Verdict::Blocked;
    }
    if !no_priority(ctx, &a) {
        return Verdict::Yielding;
    }
    Verdict::Pass
}

pub fn signal_allows(ctx: &RuleContext, slot: u32) -> bool {
    approach(ctx, slot).is_some_and(|a| allows(ctx, &a))
}

pub fn stop_satisfied(ctx: &RuleContext, slot: u32) -> bool {
    approach(ctx, slot).is_some_and(|a| stop_ok(ctx, &a))
}

pub fn has_space(ctx: &RuleContext, slot: u32) -> bool {
    approach(ctx, slot).is_some_and(|a| space_ok(ctx, &a))
}

pub fn conflicts_clear(ctx: &RuleContext, slot: u32) -> bool {
    approach(ctx, slot).is_some_and(|a| clear(ctx, &a))
}

pub fn no_priority_approach(ctx: &RuleContext, slot: u32) -> bool {
    approach(ctx, slot).is_some_and(|a| no_priority(ctx, &a))
}

fn passes_hard_rules(ctx: &RuleContext, a: &Approach) -> bool {
    allows(ctx, a) && stop_ok(ctx, a) && space_ok(ctx, a)
}

fn allows(ctx: &RuleContext, a: &Approach) -> bool {
    match ctx.network.signal_state(a.link, ctx.tick) {
        None | Some(SignalState::Green) => true,
        Some(SignalState::Red) => false,
        Some(SignalState::Amber) => a.span_end - a.s <= stopping_distance(a.v),
    }
}

fn stop_ok(ctx: &RuleContext, a: &Approach) -> bool {
    let control = ctx.network.nodes.control[a.node as usize];
    let must_stop =
        control == Control::AllWayStop || (control == Control::Stop && is_minor(ctx.network, a));
    !must_stop || ctx.vehicles.stopped_at_line[a.slot as usize]
}

fn clear(ctx: &RuleContext, a: &Approach) -> bool {
    a.junction.conflicts[usize::from(a.index)]
        .iter()
        .all(|c| movement_clear(ctx, a.node, c))
}

fn movement_clear(ctx: &RuleContext, node: u32, conflict: &Conflict) -> bool {
    ctx.occupancy
        .range(movement_key(node, conflict.other))
        .iter()
        .all(|entry| {
            entry.s >= conflict.s_other + ctx.vehicles.length(entry.slot) + CONFLICT_CLEAR_MARGIN
        })
}

fn no_priority(ctx: &RuleContext, a: &Approach) -> bool {
    a.junction.conflicts[usize::from(a.index)]
        .iter()
        .all(|c| !yields_on(ctx, a, c))
}

fn yields_on(ctx: &RuleContext, a: &Approach, conflict: &Conflict) -> bool {
    let other = &a.junction.movements[usize::from(conflict.other)];
    (0..ctx.network.link_lanes(other.from_link))
        .filter_map(|lane| candidate(ctx, other, lane))
        .any(|slot| must_yield(ctx, a, conflict, slot))
}

fn candidate(ctx: &RuleContext, other: &Movement, lane: u8) -> Option<u32> {
    ctx.occupancy
        .range(link_key(other.from_link, lane))
        .iter()
        .take(APPROACH_SCAN)
        .map(|entry| entry.slot)
        .find(|&slot| heads_to(ctx.vehicles, slot, other.to_link))
}

fn contends(ctx: &RuleContext, b: &Approach, a: &Approach) -> bool {
    allows(ctx, b) && stop_ok(ctx, b) && has_room_if_deciding(ctx, b) && beats(ctx, b, a)
}

fn has_room_if_deciding(ctx: &RuleContext, b: &Approach) -> bool {
    predecessor_committed(ctx, b)
        && (!in_zone(ctx.network, ctx.vehicles, b.slot) || room_ok(ctx, b))
}

fn must_yield(ctx: &RuleContext, a: &Approach, conflict: &Conflict, slot: u32) -> bool {
    if excluded(ctx, slot) {
        return false;
    }
    let Some(b) = approach(ctx, slot) else {
        return false;
    };
    if !ctx.vehicles.committed[slot as usize] && !contends(ctx, &b, a) {
        return false;
    }
    let t_other = approach_time(ctx, &b, b.span_end - b.s + conflict.s_other);
    let own = ctx.vehicles.length(a.slot) + CONFLICT_CLEAR_MARGIN;
    let t_self = approach_time(ctx, a, a.span_end - a.s + conflict.s_self + own);
    t_other <= t_self + courtesy_gap(ctx, a)
}

fn courtesy_gap(ctx: &RuleContext, a: &Approach) -> f64 {
    if u64::from(ctx.vehicles.wait_ticks[a.slot as usize]) >= WAIT_TIMEOUT_TICKS {
        return 0.0;
    }
    if is_minor(ctx.network, a) {
        return MINOR_GAP;
    }
    PRIORITY_GAP
}
