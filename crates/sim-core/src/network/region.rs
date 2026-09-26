use crate::geom::Vec2;

use super::Network;
use super::link::{Direction, LinkId, link_id};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Region {
    pub center: Vec2,
    pub radius: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RegionMask {
    pub circle: Region,
    pub active_road: Vec<bool>,
    pub boundary: Vec<u32>,
    pub sources: Vec<LinkId>,
    pub sinks: Vec<LinkId>,
}

impl RegionMask {
    pub fn build(network: &Network, region: Region) -> RegionMask {
        let roads = &network.roads;
        let mut active_road = vec![false; roads.count()];
        for road in network
            .spatial
            .roads_near(&network.roads, region.center, region.radius)
        {
            active_road[road as usize] = roads.is_live(road);
        }
        let boundary = boundary_nodes(network, &active_road);
        let (sources, sinks) = boundary_links(network, &active_road, &boundary);
        RegionMask {
            circle: region,
            active_road,
            boundary,
            sources,
            sinks,
        }
    }
}

fn boundary_nodes(network: &Network, active_road: &[bool]) -> Vec<u32> {
    (0..network.nodes.count() as u32)
        .filter(|&node| straddles(network, active_road, node))
        .collect()
}

fn straddles(network: &Network, active_road: &[bool], node: u32) -> bool {
    let incident = &network.nodes.roads[node as usize];
    let has_active = incident.iter().any(|&r| active_road[r as usize]);
    let has_outside = incident
        .iter()
        .any(|&r| !active_road[r as usize] && network.roads.is_live(r));
    has_active && has_outside
}

fn boundary_links(
    network: &Network,
    active_road: &[bool],
    boundary: &[u32],
) -> (Vec<LinkId>, Vec<LinkId>) {
    let mut sources = Vec::new();
    let mut sinks = Vec::new();
    let active_links = (0..network.roads.count() as u32)
        .filter(|&road| active_road[road as usize])
        .flat_map(|road| [Direction::Forward, Direction::Backward].map(|d| link_id(road, d)))
        .filter(|&link| network.link_lanes(link) > 0);
    for link in active_links {
        if boundary.binary_search(&network.link_from(link)).is_ok() {
            sources.push(link);
        }
        if boundary.binary_search(&network.link_to(link)).is_ok() {
            sinks.push(link);
        }
    }
    (sources, sinks)
}
