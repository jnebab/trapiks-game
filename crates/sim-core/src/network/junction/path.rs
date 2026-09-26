use crate::consts::{BEZIER_SEGMENTS, LANE_WIDTH};
use crate::geom::{CubicBezier, Vec2};
use crate::network::Network;
use crate::network::link::road_of;

use super::{Candidate, Movement, TurnKind, movement_class_rank};

pub fn movement(network: &Network, candidate: &Candidate, from_lanes: (u8, u8)) -> Movement {
    let primary_lanes = primary(network, candidate, from_lanes);
    let bezier = curve(network, candidate, primary_lanes);
    let (path, cumulative) = sample(&bezier);
    let class_rank = movement_class_rank(network, road_of(candidate.from_link));
    Movement {
        from_link: candidate.from_link,
        to_link: candidate.to_link,
        kind: candidate.kind,
        from_lanes,
        primary_lanes,
        path,
        cumulative,
        length: cumulative[BEZIER_SEGMENTS],
        rank: (class_rank, candidate.kind.rank()),
    }
}

fn primary(network: &Network, candidate: &Candidate, from_lanes: (u8, u8)) -> (u8, u8) {
    let target_last = network.link_lanes(candidate.to_link).saturating_sub(1);
    match candidate.kind {
        TurnKind::Right => (from_lanes.0, 0),
        TurnKind::Left | TurnKind::UTurn => (from_lanes.1, target_last),
        TurnKind::Through => (0, 0),
    }
}

fn curve(network: &Network, candidate: &Candidate, lanes: (u8, u8)) -> CubicBezier {
    let (from, to) = (candidate.from_link, candidate.to_link);
    let (p0, _) = network.link_pose(from, candidate.s0, lanes.0);
    let (p3, _) = network.link_pose(to, candidate.s3, lanes.1);
    let mut h = 0.4 * p0.distance(p3);
    if candidate.kind == TurnKind::UTurn {
        h = h.max(2.0 * LANE_WIDTH);
    }
    CubicBezier {
        p0,
        p1: p0 + candidate.d0 * h,
        p2: p3 - candidate.d3 * h,
        p3,
    }
}

type Samples = ([Vec2; BEZIER_SEGMENTS + 1], [f64; BEZIER_SEGMENTS + 1]);

fn sample(bezier: &CubicBezier) -> Samples {
    let mut path = [Vec2::default(); BEZIER_SEGMENTS + 1];
    for (i, slot) in path.iter_mut().enumerate() {
        *slot = bezier.point(i as f64 / BEZIER_SEGMENTS as f64);
    }
    let mut cumulative = [0.0; BEZIER_SEGMENTS + 1];
    for i in 1..=BEZIER_SEGMENTS {
        cumulative[i] = cumulative[i - 1] + path[i - 1].distance(path[i]);
    }
    (path, cumulative)
}
