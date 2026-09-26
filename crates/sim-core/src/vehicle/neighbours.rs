use crate::network::LinkId;

use super::ahead::Ahead;
use super::approach::RuleContext;
use super::lanes::landing_rule;
use super::leader::{current_ahead, gap_to, is_ahead, leader_beyond};
use super::occupancy::movement_index_of;
use super::{Entry, Place, link_key, movement_of};

#[derive(Clone, Copy)]
pub struct Probe {
    pub slot: u32,
    pub link: LinkId,
    pub lane: u8,
    pub s: f64,
}

#[derive(Clone, Copy)]
pub struct Follower {
    pub slot: u32,
    pub dist: f64,
    pub linked: bool,
}

pub fn leader_on(ctx: &RuleContext, probe: &Probe) -> Option<(f64, f64)> {
    let key = link_key(probe.link, probe.lane);
    let range = ctx.occupancy.range(key);
    let split = range.partition_point(|e| is_ahead(e.s, e.slot, probe.s, probe.slot));
    let pending = ctx
        .occupancy
        .pending_on(key)
        .filter(|e| is_ahead(e.s, e.slot, probe.s, probe.slot));
    let nearest = range[..split]
        .last()
        .into_iter()
        .chain(pending)
        .min_by(|a, b| a.s.total_cmp(&b.s).then_with(|| b.slot.cmp(&a.slot)));
    if let Some(entry) = nearest {
        return Some(gap_to(ctx.vehicles, probe.s, entry.slot, entry.s));
    }
    leader_beyond_link(ctx, probe)
}

fn leader_beyond_link(ctx: &RuleContext, probe: &Probe) -> Option<(f64, f64)> {
    let cached = current_ahead(ctx.network, ctx.vehicles, probe.slot);
    let landing = target_landing(ctx, probe, &cached);
    leader_beyond(ctx.vehicles, ctx.occupancy, &cached, probe.s, landing)
}

fn target_landing(ctx: &RuleContext, probe: &Probe, cached: &Ahead) -> Option<u64> {
    let (node, index) = cached.movement()?;
    let next = cached.next()?;
    let movement = movement_of(ctx.network, node, index)?;
    let route = ctx.vehicles.route(probe.slot);
    let cursor = ctx.vehicles.cursor(probe.slot);
    let rule = landing_rule(ctx.network, movement, route, cursor + 1)?;
    Some(link_key(next, rule.lane(probe.lane)))
}

pub fn follower_on(ctx: &RuleContext, probe: &Probe) -> Option<Follower> {
    linked_follower(ctx, probe).or_else(|| landing_follower(ctx, probe))
}

fn linked_follower(ctx: &RuleContext, probe: &Probe) -> Option<Follower> {
    let key = link_key(probe.link, probe.lane);
    let range = ctx.occupancy.range(key);
    let split = range.partition_point(|e| is_ahead(e.s, e.slot, probe.s, probe.slot));
    let behind = |e: &&Entry| e.slot != probe.slot && !is_ahead(e.s, e.slot, probe.s, probe.slot);
    let sorted = range[split..].iter().find(behind);
    let pending = ctx.occupancy.pending_on(key).filter(behind);
    let nearest = sorted
        .into_iter()
        .chain(pending)
        .max_by(|a, b| a.s.total_cmp(&b.s).then_with(|| b.slot.cmp(&a.slot)))?;
    Some(Follower {
        slot: nearest.slot,
        dist: probe.s - nearest.s,
        linked: true,
    })
}

fn landing_follower(ctx: &RuleContext, probe: &Probe) -> Option<Follower> {
    let node = ctx.network.link_from(probe.link);
    let junction = ctx.network.junction(node)?;
    let span_start = ctx.network.link_span(probe.link).0;
    ctx.occupancy
        .at_node(node)
        .iter()
        .filter(|entry| lands_on(ctx, entry.slot, probe.lane))
        .filter_map(|entry| {
            let movement = junction
                .movements
                .get(usize::from(movement_index_of(entry.key)))?;
            (movement.to_link == probe.link).then_some(Follower {
                slot: entry.slot,
                dist: probe.s - span_start + movement.length - entry.s,
                linked: false,
            })
        })
        .min_by(|a, b| a.dist.total_cmp(&b.dist).then_with(|| a.slot.cmp(&b.slot)))
}

fn lands_on(ctx: &RuleContext, slot: u32, lane: u8) -> bool {
    matches!(
        ctx.vehicles.place[slot as usize],
        Place::Movement { to_lane, .. } if to_lane == lane
    )
}
