use trapiks_sim_core::consts::{IDM_MAX_ACCEL, IDM_MIN_GAP, IDM_TIME_HEADWAY};
use trapiks_sim_core::vehicle::idm::acceleration;

const V0: f64 = 16.0;

#[test]
fn free_road_from_rest_accelerates_at_max() {
    let a = acceleration(0.0, V0, f64::INFINITY, 0.0);
    assert!((a - IDM_MAX_ACCEL).abs() < 1e-12);
}

#[test]
fn free_road_at_desired_speed_is_steady() {
    assert!(acceleration(V0, V0, f64::INFINITY, 0.0).abs() < 1e-12);
}

#[test]
fn standing_leader_close_ahead_brakes_hard() {
    assert!(acceleration(15.0, V0, 10.0, 15.0) < -2.0);
}

#[test]
fn equilibrium_gap_is_steady() {
    let v = V0 / 2.0;
    let r = v / V0;
    let desired = IDM_MIN_GAP + v * IDM_TIME_HEADWAY;
    let gap = desired / (1.0 - r * r * r * r).sqrt();
    assert!(acceleration(v, V0, gap, 0.0).abs() < 0.1);
}
