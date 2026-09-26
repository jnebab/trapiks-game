use std::f64::consts::TAU;

use crate::network::Network;

use super::super::apply::Edit;
use super::super::validate::{Planned, require};
use super::super::{EditError, Prepared};
use super::plan::{RingPlan, arms, plan, sweep};

pub const ROUNDABOUT_MIN_RADIUS: u8 = 16;
pub const ROUNDABOUT_MAX_RADIUS: u8 = 40;
pub const ROUNDABOUT_MIN_CHORD: f64 = 8.0;
const MIN_JUNCTION_DEGREE: usize = 3;
const MIN_SEPARATION_RAD: f64 = 0.3;
const ARM_MARGIN: f64 = 20.0;
const BASE_COST: i64 = 1200;
const COST_PER_METRE: f64 = 8.0;

pub fn build_roundabout(network: &Network, node: u32, radius_m: u8) -> Planned {
    require(
        (node as usize) < network.nodes.count(),
        EditError::NodeNotFound,
    )?;
    require(
        network.active_degree(node) >= MIN_JUNCTION_DEGREE,
        EditError::NotAJunction,
    )?;
    require(all_in_region(network, node), EditError::OutsideRegion)?;
    let radius_ok = (ROUNDABOUT_MIN_RADIUS..=ROUNDABOUT_MAX_RADIUS).contains(&radius_m);
    require(radius_ok, EditError::InvalidRadius)?;
    let arm_ids = arms(network, node);
    let flagged = arm_ids
        .iter()
        .any(|&road| network.roads.is_roundabout(road));
    require(!flagged, EditError::AlreadyRoundabout)?;
    let radius = f64::from(radius_m);
    let fits = arm_ids.iter().all(|&road| arm_fits(network, road, radius));
    require(fits, EditError::RoundaboutTooLarge)?;
    let layer = network.roads.layer[arm_ids[0] as usize];
    let level = arm_ids
        .iter()
        .all(|&road| network.roads.layer[road as usize] == layer);
    require(level, EditError::LayerMismatch)?;
    let ring = plan(network, node, radius);
    require(is_spacious(&ring), EditError::RoundaboutTooTight)?;
    Ok(Prepared::Edit {
        edit: Edit::BuildRoundabout { node, radius_m },
        cost: cost(&ring),
    })
}

fn all_in_region(network: &Network, node: u32) -> bool {
    network.nodes.roads[node as usize]
        .iter()
        .filter(|&&road| network.roads.is_live(road))
        .all(|&road| network.is_road_active(road))
}

fn arm_fits(network: &Network, road: u32, radius: f64) -> bool {
    let index = road as usize;
    network.roads.from[index] != network.roads.to[index]
        && network.roads.length[index] >= radius + ARM_MARGIN
}

fn is_spacious(ring: &RingPlan) -> bool {
    ring.pairs().all(|(i, j)| {
        let (a, b) = (&ring.arms[i], &ring.arms[j]);
        let delta = sweep(a, b);
        let separation = delta.min(TAU - delta);
        a.point.distance(b.point) >= ROUNDABOUT_MIN_CHORD && separation >= MIN_SEPARATION_RAD
    })
}

fn cost(ring: &RingPlan) -> i64 {
    BASE_COST + (COST_PER_METRE * ring.ring_length()).round() as i64
}
