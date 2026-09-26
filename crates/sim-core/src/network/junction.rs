mod conflicts;
mod lanes;
mod path;
mod rank;
mod turn;

use crate::consts::BEZIER_SEGMENTS;
use crate::geom::Vec2;

use super::Network;
use super::link::{LinkId, reverse, road_of};

pub use rank::{RING_RANK, movement_class_rank, movement_is_minor};
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
    pub has_crossing: Vec<bool>,
    pairs: Vec<(LinkId, LinkId)>,
}

impl Junction {
    pub fn movement_index(&self, from: LinkId, to: LinkId) -> Option<usize> {
        self.pairs.binary_search(&(from, to)).ok()
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

pub fn turns(network: &Network, node: u32) -> Vec<(LinkId, LinkId, TurnKind)> {
    summarize(&candidates(network, node, Bans::Respect))
}

pub fn turns_ignoring_bans(network: &Network, node: u32) -> Vec<(LinkId, LinkId, TurnKind)> {
    summarize(&candidates(network, node, Bans::Ignore))
}

fn summarize(candidates: &[Candidate]) -> Vec<(LinkId, LinkId, TurnKind)> {
    candidates
        .iter()
        .map(|c| (c.from_link, c.to_link, c.kind))
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Bans {
    Respect,
    Ignore,
}

pub fn build_junction(network: &Network, node: u32) -> Junction {
    let candidates = candidates(network, node, Bans::Respect);
    let from_lanes = lanes::assign(network, &candidates);
    let movements: Vec<Movement> = candidates
        .iter()
        .zip(from_lanes)
        .map(|(candidate, lanes)| path::movement(network, candidate, lanes))
        .collect();
    let conflicts = conflicts::find(&movements);
    Junction {
        node,
        pairs: movements.iter().map(|m| (m.from_link, m.to_link)).collect(),
        movements,
        has_crossing: conflicts
            .iter()
            .map(|list| list.iter().any(|c| !c.merge))
            .collect(),
        conflicts,
    }
}

fn candidates(network: &Network, node: u32, bans: Bans) -> Vec<Candidate> {
    let outgoing: Vec<LinkId> = network.outgoing(node).collect();
    let dead_end = network.active_degree(node) == 1;
    let mut result = Vec::new();
    for from_link in network.incoming(node) {
        for &to_link in &outgoing {
            if to_link == reverse(from_link) && !dead_end {
                continue;
            }
            if bans == Bans::Respect
                && network.is_banned(node, road_of(from_link), road_of(to_link))
            {
                continue;
            }
            result.push(candidate(network, node, from_link, to_link));
        }
    }
    result
}

fn ring_kind(network: &Network, from_link: LinkId, to_link: LinkId) -> Option<TurnKind> {
    let from_ring = network.roads.is_roundabout(road_of(from_link));
    let to_ring = network.roads.is_roundabout(road_of(to_link));
    match (from_ring, to_ring) {
        (true, true) => Some(TurnKind::Through),
        (false, false) => None,
        _ => Some(TurnKind::Right),
    }
}

fn candidate(network: &Network, node: u32, from_link: LinkId, to_link: LinkId) -> Candidate {
    let s0 = network.link_length(from_link) - network.setback_at(node, road_of(from_link));
    let s3 = network.setback_at(node, road_of(to_link));
    let (_, d0) = network.centre_pose(from_link, s0);
    let (_, d3) = network.centre_pose(to_link, s3);
    let kind = ring_kind(network, from_link, to_link)
        .unwrap_or_else(|| turn::classify(from_link, to_link, d0, d3));
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
