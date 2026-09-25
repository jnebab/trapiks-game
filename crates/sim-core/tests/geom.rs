use std::f64::consts::FRAC_PI_2;

use trapiks_sim_core::geom::{
    CubicBezier, Vec2, cumulative_lengths, intersect, pose_at, signed_turn,
};

const EAST: Vec2 = Vec2::new(1.0, 0.0);
const SOUTH: Vec2 = Vec2::new(0.0, 1.0);
const NORTH: Vec2 = Vec2::new(0.0, -1.0);

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn perp_right_of_east_is_south() {
    assert_eq!(EAST.perp_right(), SOUTH);
}

#[test]
fn signed_turn_is_positive_for_right_turns() {
    assert!(close(signed_turn(EAST, SOUTH), FRAC_PI_2));
    assert!(close(signed_turn(EAST, NORTH), -FRAC_PI_2));
}

#[test]
fn bezier_endpoints_are_exact() {
    let bezier = CubicBezier {
        p0: Vec2::new(1.3, -2.7),
        p1: Vec2::new(4.0, 5.0),
        p2: Vec2::new(-3.0, 8.0),
        p3: Vec2::new(9.1, 0.3),
    };
    let samples = bezier.sample(8);
    assert_eq!(samples.len(), 9);
    assert_eq!(samples[0], bezier.p0);
    assert_eq!(samples[8], bezier.p3);
}

#[test]
fn segments_cross() {
    let hit = intersect(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 2.0),
        Vec2::new(0.0, 2.0),
        Vec2::new(2.0, 0.0),
    );
    let (t, u) = hit.expect("crossing");
    assert!(close(t, 0.5) && close(u, 0.5));
}

#[test]
fn parallel_segments_do_not_intersect() {
    let hit = intersect(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(2.0, 1.0),
    );
    assert_eq!(hit, None);
}

#[test]
fn segments_touching_at_endpoint_intersect() {
    let hit = intersect(
        Vec2::new(0.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(2.0, 0.0),
        Vec2::new(2.0, 3.0),
    );
    let (t, u) = hit.expect("touching");
    assert!(close(t, 1.0) && close(u, 0.0));
}

#[test]
fn pose_at_walks_the_polyline() {
    let points = [
        Vec2::new(0.0, 0.0),
        Vec2::new(10.0, 0.0),
        Vec2::new(10.0, 10.0),
    ];
    let cumulative = cumulative_lengths(&points);
    assert_eq!(cumulative, vec![0.0, 10.0, 20.0]);
    assert_eq!(pose_at(&points, &cumulative, 0.0), (points[0], EAST));
    assert_eq!(
        pose_at(&points, &cumulative, 5.0),
        (Vec2::new(5.0, 0.0), EAST)
    );
    assert_eq!(
        pose_at(&points, &cumulative, 15.0),
        (Vec2::new(10.0, 5.0), SOUTH)
    );
    assert_eq!(pose_at(&points, &cumulative, 20.0), (points[2], SOUTH));
    assert_eq!(pose_at(&points, &cumulative, 99.0), (points[2], SOUTH));
    assert_eq!(pose_at(&points, &cumulative, -5.0), (points[0], EAST));
}
