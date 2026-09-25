use std::f64::consts::PI;

use crate::consts::SIGNAL_BIN_TOLERANCE_DEG;
use crate::geom::axis_angle;
use crate::network::link::{LinkId, centre_pose, length};
use crate::network::road::RoadStore;

const BIN_B_MIN_DEG: f64 = 90.0 - SIGNAL_BIN_TOLERANCE_DEG;

pub fn plan(roads: &RoadStore, approaches: &[LinkId]) -> Vec<Vec<LinkId>> {
    let Some(&first) = approaches.first() else {
        return Vec::new();
    };
    two_bins(roads, approaches, first).unwrap_or_else(|| one_per_approach(approaches))
}

fn two_bins(roads: &RoadStore, approaches: &[LinkId], first: LinkId) -> Option<Vec<Vec<LinkId>>> {
    let a0 = end_axis(roads, first);
    let mut bin_a = Vec::new();
    let mut bin_b = Vec::new();
    for &link in approaches {
        let delta = axis_delta(end_axis(roads, link), a0).to_degrees();
        if delta <= SIGNAL_BIN_TOLERANCE_DEG {
            bin_a.push(link);
        } else if delta >= BIN_B_MIN_DEG {
            bin_b.push(link);
        } else {
            return None;
        }
    }
    (!bin_b.is_empty()).then(|| vec![bin_a, bin_b])
}

fn one_per_approach(approaches: &[LinkId]) -> Vec<Vec<LinkId>> {
    approaches.iter().map(|&l| vec![l]).collect()
}

fn end_axis(roads: &RoadStore, link: LinkId) -> f64 {
    let (_, tangent) = centre_pose(roads, link, length(roads, link));
    axis_angle(tangent)
}

fn axis_delta(a: f64, a0: f64) -> f64 {
    let d = (a - a0).abs() % PI;
    d.min(PI - d)
}
