mod support;

use support::{Must, check_invariants, run_until, slot_of};
use trapiks_sim_core::consts::IDM_MIN_GAP;
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::{Place, VehicleKind};

const DRAWS: u32 = 10_000;
const TOLERANCE_PP: f64 = 1.5;

#[test]
fn kinds_spawn_shares() {
    let mut rng = Pcg32::new(42, 3);
    let mut counts = [0u32; 3];
    for _ in 0..DRAWS {
        counts[usize::from(VehicleKind::draw(&mut rng).code())] += 1;
    }
    for (count, expected) in counts.iter().zip([80.0, 15.0, 5.0]) {
        let share = f64::from(*count) * 100.0 / f64::from(DRAWS);
        assert!((share - expected).abs() <= TOLERANCE_PP, "{counts:?}");
    }
}

fn spawn_car_behind(sim: &mut Sim, bus: u32, span_start: f64) -> u32 {
    for _ in 0..600 {
        let bus_slot = slot_of(sim, bus).must("bus alive");
        let bus_s = sim.vehicles().s[bus_slot as usize];
        if let Ok(car) = sim.spawn(&[0, 5]) {
            assert!(bus_s - VehicleKind::Bus.length() - span_start >= IDM_MIN_GAP);
            return car;
        }
        sim.step();
    }
    panic!("car never spawned");
}

#[test]
fn bus_following() {
    let mut sim = Sim::new(&four_way(1, 200.0), 1);
    let bus = sim.spawn_kind(&[0, 5], VehicleKind::Bus).must("bus");
    let span_start = sim.network().link_span(0).0;
    let car = spawn_car_behind(&mut sim, bus, span_start);
    let mut car_reached = false;
    let done = run_until(&mut sim, 3_000, |sim| {
        check_invariants(sim);
        car_reached |= slot_of(sim, car).is_some_and(|slot| {
            sim.vehicles().place[slot as usize] != Place::Link { link: 0, lane: 0 }
        });
        slot_of(sim, car).is_none()
    });
    assert!(done && car_reached);
}
