mod conflicts;
mod lanes;
mod path;
mod turn;

use crate::consts::BEZIER_SEGMENTS;
use crate::geom::Vec2;

use super::Network;
use super::link::{LinkId, reverse, road_of};

pub use turn::TurnKind;

#[derive(Clone, Debug, PartialEq)]
pub struct Movement {
    pub from_link: LinkId,
    pub to_link: LinkId,
    pub kind: TurnKind,
    pub from_lanes: (u8, u8),
    pub primary_lanes: (u8, u8),
    pub path: [Vec2; BEZIER_SEGMENTS + 1],
    pub cumulative: [f64; BEZIER_SEGMENTS + 1],
    pub length: f64,
    pub rank: (u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Conflict {
    pub other: u16,
    pub s_self: f64,
    pub s_other: f64,
    pub merge: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Junction {
    pub node: u32,
    pub movements: Vec<Movement>,
    pub conflicts: Vec<Vec<Conflict>>,
}

impl Junction {
    pub fn movement_index(&self, from: LinkId, to: LinkId) -> Option<usize> {
        self.movements
            .binary_search_by_key(&(from, to), |m| (m.from_link, m.to_link))
            .ok()
    }
}

#[derive(Clone, Copy, Debug)]
struct Candidate {
    from_link: LinkId,
    to_link: LinkId,
    kind: TurnKind,
    d0: Vec2,
    d3: Vec2,
    s0: f64,
    s3: f64,
}

pub fn build_junction(network: &Network, node: u32) -> Junction {
    let candidates = candidates(network, node);
    let from_lanes = lanes::assign(network, &candidates);
    let movements: Vec<Movement> = candidates
        .iter()
        .zip(from_lanes)
        .map(|(candidate, lanes)| path::movement(network, candidate, lanes))
        .collect();
    let conflicts = conflicts::find(&movements);
    Junction {
        node,
        movements,
        conflicts,
    }
}

fn candidates(network: &Network, node: u32) -> Vec<Candidate> {
    let outgoing: Vec<LinkId> = network.outgoing(node).collect();
    let dead_end = network.active_degree(node) == 1;
    let mut result = Vec::new();
    for from_link in network.incoming(node) {
        for &to_link in &outgoing {
            if to_link == reverse(from_link) && !dead_end {
                continue;
            }
            if network.is_banned(node, road_of(from_link), road_of(to_link)) {
                continue;
            }
            result.push(candidate(network, node, from_link, to_link));
        }
    }
    result
}

fn candidate(network: &Network, node: u32, from_link: LinkId, to_link: LinkId) -> Candidate {
    let s0 = network.link_length(from_link) - network.setback_at(node, road_of(from_link));
    let s3 = network.setback_at(node, road_of(to_link));
    let (_, d0) = network.centre_pose(from_link, s0);
    let (_, d3) = network.centre_pose(to_link, s3);
    let kind = turn::classify(from_link, to_link, d0, d3);
    Candidate {
        from_link,
        to_link,
        kind,
        d0,
        d3,
        s0,
        s3,
    }
}
