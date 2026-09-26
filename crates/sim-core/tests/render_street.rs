use trapiks_sim_core::fixtures::{four_way, t_junction};
use trapiks_sim_core::network::Network;
use trapiks_sim_core::render::{
    FILLET_RADIUS_MAX, JunctionShapes, MARKER_SIGNAL, MARKER_YIELD, approach_markers,
    junction_shapes, road_setbacks,
};

const FILLET_POINTS: usize = 7;

fn four_way_network() -> Network {
    Network::from_map(&four_way(2, 200.0))
}

fn ring(shapes: &JunctionShapes) -> Vec<(f64, f64)> {
    shapes
        .x
        .iter()
        .zip(&shapes.y)
        .map(|(&x, &y)| (f64::from(x), f64::from(y)))
        .collect()
}

fn cross(o: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

fn segments_cross(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    let d1 = cross(c, d, a);
    let d2 = cross(c, d, b);
    let d3 = cross(a, b, c);
    let d4 = cross(a, b, d);
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

fn self_intersects(points: &[(f64, f64)]) -> bool {
    let n = points.len();
    let edge = |i: usize| (points[i], points[(i + 1) % n]);
    (0..n).any(|i| {
        (i + 2..n).filter(|&j| (j + 1) % n != i).any(|j| {
            let (a, b) = edge(i);
            let (c, d) = edge(j);
            segments_cross(a, b, c, d)
        })
    })
}

#[test]
fn four_way_has_one_shape_of_28_points() {
    let network = four_way_network();
    let shapes = junction_shapes(&network);
    assert_eq!(shapes.node, vec![0]);
    assert_eq!(shapes.ring_start, vec![0, 28]);
    assert_eq!(shapes.x.len(), 28);
    let half_width = network.roads.width(0) / 2.0;
    let reach = network.setback_at(0, 0) + FILLET_RADIUS_MAX + 2.0 * half_width;
    for (x, y) in ring(&shapes) {
        assert!(x.hypot(y) <= reach);
    }
}

#[test]
fn four_way_ring_is_simple() {
    let shapes = junction_shapes(&four_way_network());
    assert!(!self_intersects(&ring(&shapes)));
}

#[test]
fn fillet_is_concave() {
    let shapes = junction_shapes(&four_way_network());
    let points = ring(&shapes);
    let middle = points
        .chunks(FILLET_POINTS)
        .map(|fillet| fillet[FILLET_POINTS / 2])
        .find(|p| p.0 > 0.0 && p.1 < 0.0)
        .unwrap_or_default();
    assert!((middle.0 - 8.15).abs() < 1e-3);
    assert!((middle.1 + 8.15).abs() < 1e-3);
    assert!(middle.0.hypot(middle.1) > 6.4_f64.hypot(6.4));
}

#[test]
fn four_way_has_four_signal_markers() {
    let markers = approach_markers(&four_way_network());
    assert_eq!(markers.kind, vec![MARKER_SIGNAL; 4]);
}

#[test]
fn t_junction_has_one_yield_marker() {
    let markers = approach_markers(&Network::from_map(&t_junction()));
    assert_eq!(markers.kind, vec![MARKER_YIELD]);
    assert_eq!(markers.link, vec![4]);
}

#[test]
fn marker_length_matches_lane_count() {
    for map in [four_way(2, 200.0), four_way(3, 200.0), t_junction()] {
        let network = Network::from_map(&map);
        let markers = approach_markers(&network);
        for i in 0..markers.link.len() {
            let dx = f64::from(markers.x2[i] - markers.x1[i]);
            let dy = f64::from(markers.y2[i] - markers.y1[i]);
            let lanes = f64::from(network.link_lanes(markers.link[i]));
            assert!((dx.hypot(dy) - lanes * 3.2).abs() < 1e-3);
        }
    }
}

#[test]
fn setbacks_are_zero_away_from_junctions() {
    let network = four_way_network();
    let setbacks = road_setbacks(&network);
    assert_eq!(setbacks.len(), 8);
    for road in 0..4 {
        assert_eq!(setbacks[road * 2], 0.0);
        assert!((f64::from(setbacks[road * 2 + 1]) - 7.4).abs() < 1e-4);
    }
}

#[test]
fn render_data_is_deterministic() {
    let a = four_way_network();
    let b = four_way_network();
    assert_eq!(junction_shapes(&a), junction_shapes(&b));
    assert_eq!(approach_markers(&a), approach_markers(&b));
    assert_eq!(road_setbacks(&a), road_setbacks(&b));
}
