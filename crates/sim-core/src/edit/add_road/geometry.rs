use crate::geom::{QuadraticBezier, Vec2, split_polyline};
use crate::map::RoadClass;
use crate::network::Network;

use super::super::Endpoint;

pub const ADD_ROAD_SEGMENTS: usize = 16;
const BASE_COST: f64 = 200.0;
const COST_PER_LANE_METRE: f64 = 2.0;
const SPLIT_COST: i64 = 200;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AddRoadSpec {
    pub from: Endpoint,
    pub to: Endpoint,
    pub via: Option<[f64; 2]>,
    pub lanes: (u8, u8),
    pub layer: i8,
}

impl AddRoadSpec {
    pub fn ends(&self) -> [Endpoint; 2] {
        [self.from, self.to]
    }

    pub fn split_count(&self) -> usize {
        self.ends()
            .iter()
            .filter(|end| matches!(end, Endpoint::OnRoad { .. }))
            .count()
    }
}

pub fn cut_road(network: &Network, road: u32, at_m: f64) -> (Vec<Vec2>, Vec<Vec2>) {
    split_polyline(
        network.roads.points(road),
        network.roads.cumulative(road),
        at_m,
    )
}

pub fn resolve(network: &Network, endpoint: Endpoint) -> Vec2 {
    match endpoint {
        Endpoint::Node { node } => network.nodes.pos[node as usize],
        Endpoint::OnRoad { road, at_m } => {
            let (head, _) = cut_road(network, road, at_m);
            head.last().copied().unwrap_or_default()
        }
    }
}

pub fn curve_points(from: Vec2, to: Vec2, via: Option<Vec2>) -> Vec<Vec2> {
    let Some(via) = via else {
        return vec![from, to];
    };
    let curve = QuadraticBezier {
        p0: from,
        p1: via,
        p2: to,
    };
    let mut points: Vec<Vec2> = (0..=ADD_ROAD_SEGMENTS)
        .map(|k| curve.point(k as f64 / ADD_ROAD_SEGMENTS as f64))
        .collect();
    points[0] = from;
    points[ADD_ROAD_SEGMENTS] = to;
    points
}

pub fn via_point(spec: &AddRoadSpec) -> Option<Vec2> {
    spec.via.map(|[x, y]| Vec2::new(x, y))
}

pub fn plan_points(network: &Network, spec: &AddRoadSpec) -> Vec<Vec2> {
    curve_points(
        resolve(network, spec.from),
        resolve(network, spec.to),
        via_point(spec),
    )
}

pub fn polyline_length(points: &[Vec2]) -> f64 {
    points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum()
}

pub fn class_and_speed((forward, backward): (u8, u8)) -> (RoadClass, u8) {
    match forward.max(backward) {
        0 | 1 => (RoadClass::Residential, 40),
        2 => (RoadClass::Tertiary, 50),
        _ => (RoadClass::Primary, 60),
    }
}

pub fn cost(spec: &AddRoadSpec, length: f64) -> i64 {
    let lanes = f64::from(spec.lanes.0) + f64::from(spec.lanes.1);
    let elevation = 1.0 + f64::from(spec.layer.clamp(0, 1));
    let road = ((BASE_COST + COST_PER_LANE_METRE * length * lanes) * elevation).round() as i64;
    road + SPLIT_COST * spec.split_count() as i64
}
