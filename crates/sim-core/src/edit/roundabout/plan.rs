use std::f64::consts::TAU;

use crate::geom::{Vec2, split_polyline};
use crate::map::RoadClass;
use crate::network::{Network, reverse};

const ARC_STEP: f64 = 4.0;
const MIN_ARC_SEGMENTS: usize = 4;

#[derive(Clone, Debug, PartialEq)]
pub struct ArmCut {
    pub road: u32,
    pub arrived: bool,
    pub outer: Vec<Vec2>,
    pub point: Vec2,
    pub theta: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RingPlan {
    pub arms: Vec<ArmCut>,
    pub order: Vec<usize>,
    pub arcs: Vec<Vec<Vec2>>,
    pub lanes: u8,
    pub class: RoadClass,
    pub layer: i8,
}

impl RingPlan {
    pub fn ring_length(&self) -> f64 {
        self.arcs.iter().map(|arc| polyline_length(arc)).sum()
    }

    pub fn pairs(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        let count = self.order.len();
        (0..count).map(move |k| (self.order[k], self.order[(k + 1) % count]))
    }
}

pub fn arms(network: &Network, node: u32) -> Vec<u32> {
    network.nodes.roads[node as usize]
        .iter()
        .copied()
        .filter(|&road| network.is_road_active(road))
        .collect()
}

pub fn plan(network: &Network, node: u32, radius: f64) -> RingPlan {
    let centre = network.nodes.pos[node as usize];
    let arm_ids = arms(network, node);
    let arms: Vec<ArmCut> = arm_ids
        .iter()
        .map(|&road| cut_arm(network, node, road, radius))
        .collect();
    let order = ring_order(&arms, &arm_ids);
    let mut plan = RingPlan {
        order,
        arcs: Vec::new(),
        lanes: ring_lanes(network, node, &arm_ids),
        class: top_class(network, &arm_ids),
        layer: network.roads.layer[arm_ids.first().copied().unwrap_or_default() as usize],
        arms,
    };
    plan.arcs = plan
        .pairs()
        .map(|(i, j)| arc(centre, &plan.arms[i], &plan.arms[j], radius))
        .collect();
    plan
}

fn cut_arm(network: &Network, node: u32, road: u32, radius: f64) -> ArmCut {
    let arrived = network.roads.to[road as usize] == node;
    let length = network.roads.length[road as usize];
    let cut_s = if arrived { length - radius } else { radius };
    let (head, tail) = split_polyline(
        network.roads.points(road),
        network.roads.cumulative(road),
        cut_s,
    );
    let outer = if arrived { head } else { tail };
    let point = if arrived { outer.last() } else { outer.first() };
    let point = point.copied().unwrap_or_default();
    let centre = network.nodes.pos[node as usize];
    ArmCut {
        road,
        arrived,
        outer,
        point,
        theta: libm::atan2(point.y - centre.y, point.x - centre.x),
    }
}

fn ring_order(arms: &[ArmCut], ids: &[u32]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..arms.len()).collect();
    order.sort_by(|&a, &b| {
        arms[b]
            .theta
            .total_cmp(&arms[a].theta)
            .then(ids[a].cmp(&ids[b]))
    });
    let start = order.iter().position(|&i| i == 0).unwrap_or_default();
    order.rotate_left(start);
    order
}

fn ring_lanes(network: &Network, node: u32, arms: &[u32]) -> u8 {
    let wide = arms.iter().any(|&road| {
        let link = network.departing_link(road, node);
        network.link_lanes(reverse(link)) >= 2
    });
    if wide { 2 } else { 1 }
}

fn top_class(network: &Network, arms: &[u32]) -> RoadClass {
    arms.iter()
        .map(|&road| network.roads.class[road as usize])
        .max_by_key(|class| class.rank())
        .unwrap_or(RoadClass::Residential)
}

pub fn sweep(from: &ArmCut, to: &ArmCut) -> f64 {
    let delta = (from.theta - to.theta).rem_euclid(TAU);
    if delta <= 0.0 { TAU } else { delta }
}

fn arc(centre: Vec2, from: &ArmCut, to: &ArmCut, radius: f64) -> Vec<Vec2> {
    let delta = sweep(from, to);
    let segments = (libm::ceil(delta * radius / ARC_STEP) as usize).max(MIN_ARC_SEGMENTS);
    let (r0, r1) = (from.point.distance(centre), to.point.distance(centre));
    let mut points: Vec<Vec2> = (0..=segments)
        .map(|k| {
            let t = k as f64 / segments as f64;
            let theta = from.theta - t * delta;
            let r = r0 + t * (r1 - r0);
            centre + Vec2::new(libm::cos(theta), libm::sin(theta)) * r
        })
        .collect();
    points[0] = from.point;
    points[segments] = to.point;
    points
}

fn polyline_length(points: &[Vec2]) -> f64 {
    points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum()
}
