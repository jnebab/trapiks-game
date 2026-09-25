use trapiks_sim_core::consts::CAR_LENGTH;
use trapiks_sim_core::fixtures::corridor;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::{Place, SpawnError};

const ROUTE: [u32; 3] = [0, 2, 4];
const TARGET: usize = 30;

#[test]
fn thirty_vehicles_drive_the_corridor_and_despawn() {
    let mut sim = Sim::new(&corridor(), 1);
    let mut spawned = 0;
    let mut next_spawn = 0;
    while sim.tick() < 6_000 {
        if spawned < TARGET && sim.tick() >= next_spawn {
            match sim.spawn(&ROUTE) {
                Ok(_) => {
                    spawned += 1;
                    next_spawn = sim.tick() + 25;
                }
                Err(error) => assert_eq!(error, SpawnError::Blocked),
            }
        }
        sim.step();
        assert_invariants(&sim);
        if spawned == TARGET && sim.vehicle_count() == 0 {
            break;
        }
    }
    assert_eq!(spawned, TARGET);
    assert_eq!(sim.vehicle_count(), 0);
}

fn assert_invariants(sim: &Sim) {
    let vehicles = sim.vehicles();
    let mut on_links: Vec<(u32, u8, f64)> = Vec::new();
    for slot in 0..vehicles.alive.len() {
        if !vehicles.alive[slot] {
            continue;
        }
        let (s, v) = (vehicles.s[slot], vehicles.v[slot]);
        assert!(!s.is_nan() && !v.is_nan());
        assert!(v >= 0.0);
        if let Place::Link { link, lane } = vehicles.place[slot] {
            on_links.push((link, lane, s));
        }
    }
    on_links.sort_by(|a, b| (a.0, a.1).cmp(&(b.0, b.1)).then(a.2.total_cmp(&b.2)));
    for pair in on_links.windows(2) {
        if (pair[0].0, pair[0].1) == (pair[1].0, pair[1].1) {
            assert!(pair[1].2 - pair[0].2 - CAR_LENGTH >= 0.0);
        }
    }
}
