use crate::geom::{QuadraticBezier, Vec2};
use crate::network::Network;

use super::{FILLET_RADIUS_MAX, JUNCTION_MIN_DEGREE};

const FILLET_SEGMENTS: usize = 6;
const PARALLEL_EPSILON: f64 = 1e-9;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JunctionShapes {
    pub node: Vec<u32>,
    pub layer: Vec<i8>,
    pub min_layer: Vec<i8>,
    pub fillet_layer: Vec<i8>,
    pub ring_start: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
}

struct Arm {
    road: u32,
    angle: f64,
    dir: Vec2,
    left: Vec2,
    right: Vec2,
}

pub fn junction_shapes(network: &Network) -> JunctionShapes {
    let mut shapes = JunctionShapes::default();
    for node in 0..network.nodes.count() as u32 {
        if let Some(arms) = junction_arms(network, node) {
            push_shape(network, node, &arms, &mut shapes);
        }
    }
    shapes.ring_start.push(shapes.x.len() as u32);
    shapes
}

pub fn node_shapes(network: &Network, nodes: &[u32]) -> JunctionShapes {
    let mut shapes = JunctionShapes::default();
    for &node in nodes {
        let arms = junction_arms(network, node).unwrap_or_default();
        push_shape(network, node, &arms, &mut shapes);
    }
    shapes.ring_start.push(shapes.x.len() as u32);
    shapes
}

fn junction_arms(network: &Network, node: u32) -> Option<Vec<Arm>> {
    if network.active_degree(node) < JUNCTION_MIN_DEGREE {
        return None;
    }
    let arms = arms_of(network, node);
    (arms.len() >= 2).then_some(arms)
}

fn push_shape(network: &Network, node: u32, arms: &[Arm], shapes: &mut JunctionShapes) {
    let layer_of = |arm: &Arm| network.roads.layer[arm.road as usize];
    let layers = arms.iter().map(layer_of);
    shapes.node.push(node);
    shapes.layer.push(layers.clone().max().unwrap_or_default());
    shapes.min_layer.push(layers.min().unwrap_or_default());
    shapes.ring_start.push(shapes.x.len() as u32);
    let centre = network.nodes.pos[node as usize];
    for (i, a) in arms.iter().enumerate() {
        let b = &arms[(i + 1) % arms.len()];
        shapes.fillet_layer.push(layer_of(a).min(layer_of(b)));
        push_fillet(shapes, fillet(centre, a, b));
    }
}

fn arms_of(network: &Network, node: u32) -> Vec<Arm> {
    let mut arms: Vec<Arm> = network.nodes.roads[node as usize]
        .iter()
        .copied()
        .filter(|&road| is_arm(network, road))
        .map(|road| arm(network, node, road))
        .collect();
    arms.sort_by(|a, b| a.angle.total_cmp(&b.angle).then(a.road.cmp(&b.road)));
    arms
}

fn is_arm(network: &Network, road: u32) -> bool {
    let index = road as usize;
    network.is_road_active(road) && network.roads.from[index] != network.roads.to[index]
}

fn arm(network: &Network, node: u32, road: u32) -> Arm {
    let half_width = network.roads.width(road) / 2.0;
    let reach = network.setback_at(node, road) + FILLET_RADIUS_MAX.min(half_width);
    let t = reach.min(network.roads.length[road as usize] / 2.0);
    let (centre, dir) = network.centre_pose(network.departing_link(road, node), t);
    let right = dir.perp_right() * half_width;
    Arm {
        road,
        angle: libm::atan2(dir.y, dir.x),
        dir,
        left: centre - right,
        right: centre + right,
    }
}

fn fillet(node: Vec2, a: &Arm, b: &Arm) -> QuadraticBezier {
    let control = corner(node, a, b).unwrap_or_else(|| Vec2::lerp(a.right, b.left, 0.5));
    QuadraticBezier {
        p0: a.right,
        p1: control,
        p2: b.left,
    }
}

fn corner(node: Vec2, a: &Arm, b: &Arm) -> Option<Vec2> {
    let denominator = a.dir.cross(b.dir);
    if denominator.abs() < PARALLEL_EPSILON {
        return None;
    }
    let s = (b.left - a.right).cross(b.dir) / denominator;
    let point = a.right + a.dir * s;
    let limit = node.distance(a.right).max(node.distance(b.left));
    (node.distance(point) <= limit).then_some(point)
}

fn push_fillet(shapes: &mut JunctionShapes, curve: QuadraticBezier) {
    for i in 0..=FILLET_SEGMENTS {
        let p = curve.point(i as f64 / FILLET_SEGMENTS as f64);
        shapes.x.push(p.x as f32);
        shapes.y.push(p.y as f32);
    }
}
