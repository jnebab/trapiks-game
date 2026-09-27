mod support;

use support::apply_now;
use trapiks_sim_core::edit::{EditCommand, Outcome};
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::network::Network;
use trapiks_sim_core::render::{JunctionShapes, junction_shapes};
use trapiks_sim_core::sim::Sim;

const RADIUS: u8 = 28;
const RING_NODES: std::ops::Range<u32> = 5..9;
const INSIDE_EDGE: f64 = 0.5;
const OUTLINE: f64 = 0.35;

fn ring(shapes: &JunctionShapes, shape: usize) -> Vec<Vec2> {
    let (start, end) = (shapes.ring_start[shape], shapes.ring_start[shape + 1]);
    (start as usize..end as usize)
        .map(|i| Vec2::new(f64::from(shapes.x[i]), f64::from(shapes.y[i])))
        .collect()
}

fn contains(polygon: &[Vec2], p: Vec2) -> bool {
    let mut inside = false;
    for (i, &a) in polygon.iter().enumerate() {
        let b = polygon[(i + polygon.len() - 1) % polygon.len()];
        if (a.y > p.y) != (b.y > p.y) && p.x < a.x + (p.y - a.y) / (b.y - a.y) * (b.x - a.x) {
            inside = !inside;
        }
    }
    inside
}

fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f64 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.dot(ab)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

fn is_ring_node(network: &Network, node: u32) -> bool {
    network.nodes.roads[node as usize]
        .iter()
        .any(|&road| network.roads.is_roundabout(road))
}

fn trim_end(points: &mut [Vec2], at_end: bool, by: f64) {
    let n = points.len();
    let (tip, next) = if at_end { (n - 1, n - 2) } else { (0, 1) };
    let direction = (points[next] - points[tip]).normalized();
    points[tip] = points[tip] + direction * by;
}

fn drawn_polyline(network: &Network, road: u32) -> Vec<Vec2> {
    let mut points = network.roads.points(road).to_vec();
    if network.roads.is_roundabout(road) {
        return points;
    }
    let index = road as usize;
    for (node, at_end) in [
        (network.roads.from[index], false),
        (network.roads.to[index], true),
    ] {
        if is_ring_node(network, node) {
            trim_end(&mut points, at_end, network.setback_at(node, road));
        }
    }
    points
}

fn on_road(network: &Network, p: Vec2) -> bool {
    (0..network.roads.count() as u32)
        .filter(|&road| network.is_road_active(road))
        .any(|road| {
            let reach = network.roads.width(road) / 2.0 + OUTLINE;
            drawn_polyline(network, road)
                .windows(2)
                .any(|pair| segment_distance(p, pair[0], pair[1]) <= reach)
        })
}

#[test]
fn roundabout_island_stays_ground_colour() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let command = EditCommand::BuildRoundabout {
        node: 0,
        radius_m: RADIUS,
    };
    assert!(matches!(apply_now(&mut sim, command), Outcome::Ok(_)));
    let network = sim.network();
    let shapes = junction_shapes(network);
    let polygons: Vec<Vec<Vec2>> = (0..shapes.node.len()).map(|s| ring(&shapes, s)).collect();
    for node in RING_NODES {
        let pos = network.nodes.pos[node as usize];
        let edge = f64::from(RADIUS) - network.roads.width(4) / 2.0;
        let sample = pos.normalized() * (edge - INSIDE_EDGE);
        assert!(!on_road(network, sample), "node {node} sample on a road");
        let covered = polygons.iter().any(|polygon| contains(polygon, sample));
        assert!(!covered, "node {node} island covered by its junction shape");
    }
}
