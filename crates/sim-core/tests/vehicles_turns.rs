use std::f64::consts::PI;

use trapiks_sim_core::consts::DT;
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::sim::{Sim, Snapshot};

struct Sample {
    x: f64,
    y: f64,
    heading: f64,
    v: f64,
}

fn drive(route: &[u32]) -> Vec<Sample> {
    let mut sim = Sim::new(&four_way(2, 200.0), 3);
    assert!(sim.spawn(route).is_ok());
    let mut snapshot = Snapshot::default();
    let mut samples = Vec::new();
    for _ in 0..2_000 {
        sim.fill_snapshot(&mut snapshot);
        if snapshot.ids.is_empty() {
            return samples;
        }
        samples.push(Sample {
            x: f64::from(snapshot.x[0]),
            y: f64::from(snapshot.y[0]),
            heading: f64::from(snapshot.heading[0]),
            v: sim.vehicles().v[0],
        });
        sim.step();
    }
    panic!("vehicle did not despawn");
}

fn wrapped(delta: f64) -> f64 {
    let mut d = delta;
    while d > PI {
        d -= 2.0 * PI;
    }
    while d <= -PI {
        d += 2.0 * PI;
    }
    d
}

fn assert_continuous(samples: &[Sample]) {
    for pair in samples.windows(2) {
        let step = ((pair[1].x - pair[0].x).powi(2) + (pair[1].y - pair[0].y).powi(2)).sqrt();
        assert!(step <= pair[0].v * DT + 0.5, "jump {step}");
    }
}

#[test]
fn left_turn_is_smooth_and_despawns() {
    let samples = drive(&[0, 3]);
    assert!(samples.len() > 10);
    assert_continuous(&samples);
    for pair in samples.windows(2) {
        assert!(wrapped(pair[1].heading - pair[0].heading).abs() < 0.3);
    }
}

#[test]
fn right_turn_is_continuous_and_despawns() {
    let samples = drive(&[0, 7]);
    assert!(samples.len() > 10);
    assert_continuous(&samples);
}
