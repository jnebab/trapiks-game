use crate::network::Network;

use super::apply::Edit;
use super::flyover::reach;
use super::validate::{Planned, require};
use super::{EditError, Prepared};

const MIN_JUNCTION_DEGREE: usize = 3;
const DIRECTION_PROBE: f64 = 10.0;
const MIN_ANGLE_DEG: f64 = 150.0;
const MIN_LENGTH: f64 = 30.0;
const BASE_COST: i64 = 2000;
const COST_PER_METRE: f64 = 10.0;

pub fn build_flyover(network: &Network, node: u32, through: [u32; 2]) -> Planned {
    require(
        (node as usize) < network.nodes.count(),
        EditError::NodeNotFound,
    )?;
    require(
        network.active_degree(node) >= MIN_JUNCTION_DEGREE,
        EditError::NotAJunction,
    )?;
    let mut through = through;
    through.sort_unstable();
    let incident =
        through[0] != through[1] && through.iter().all(|&road| is_incident(network, node, road));
    require(incident, EditError::RoadNotIncident)?;
    let inside = through.iter().all(|&road| network.is_road_active(road));
    require(inside, EditError::OutsideRegion)?;
    require(
        is_straight(network, node, through),
        EditError::FlyoverNotStraight,
    )?;
    let long_enough = through
        .iter()
        .all(|&road| network.roads.length[road as usize] >= MIN_LENGTH);
    require(long_enough, EditError::FlyoverTooShort)?;
    Ok(Prepared::Edit {
        edit: Edit::BuildFlyover { node, through },
        cost: cost(network, through),
    })
}

fn is_incident(network: &Network, node: u32, road: u32) -> bool {
    let index = road as usize;
    if index >= network.roads.count() || !network.roads.is_live(road) {
        return false;
    }
    let (from, to) = (network.roads.from[index], network.roads.to[index]);
    from != to && (from == node || to == node)
}

fn is_straight(network: &Network, node: u32, through: [u32; 2]) -> bool {
    let [d0, d1] = through.map(|road| {
        let link = network.departing_link(road, node);
        network.centre_pose(link, DIRECTION_PROBE).1
    });
    d0.dot(d1) <= libm::cos(MIN_ANGLE_DEG.to_radians())
}

fn cost(network: &Network, through: [u32; 2]) -> i64 {
    let inner: f64 = through
        .iter()
        .map(|&road| reach(network.roads.length[road as usize]))
        .sum();
    BASE_COST + (COST_PER_METRE * inner).round() as i64
}
