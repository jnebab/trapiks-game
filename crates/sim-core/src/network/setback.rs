use crate::consts::{LANE_WIDTH, MAX_SETBACK_FRACTION, SETBACK_MARGIN};

use super::node::NodeStore;
use super::road::RoadStore;

pub fn setback(
    roads: &RoadStore,
    nodes: &NodeStore,
    is_active: impl Fn(u32) -> bool,
    node: u32,
    road: u32,
) -> f64 {
    let incident = &nodes.roads[node as usize];
    let degree = nodes.active_degree(node, &is_active);
    let cap = MAX_SETBACK_FRACTION * roads.length[road as usize];
    match degree {
        0 | 1 => 0.0,
        2 => LANE_WIDTH.min(cap),
        _ => widest_other(roads, incident, &is_active, road).min(cap),
    }
}

fn widest_other(
    roads: &RoadStore,
    incident: &[u32],
    is_active: &impl Fn(u32) -> bool,
    road: u32,
) -> f64 {
    let half_width = incident
        .iter()
        .copied()
        .filter(|&r| r != road && is_active(r))
        .map(|r| roads.width(r) / 2.0)
        .fold(0.0, f64::max);
    half_width + SETBACK_MARGIN
}
