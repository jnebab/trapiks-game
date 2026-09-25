use trapiks_sim_core::fixtures::corridor;
use trapiks_sim_core::sim::{Sim, Snapshot};

#[test]
fn snapshot_follows_live_slot_order() {
    let mut sim = Sim::new(&corridor(), 5);
    let mut snapshot = Snapshot::default();
    for tick in 0..400 {
        if tick % 30 == 0 {
            let _ = sim.spawn(&[0, 2, 4]);
        }
        sim.step();
        sim.fill_snapshot(&mut snapshot);
        let vehicles = sim.vehicles();
        let live: Vec<u32> = (0..vehicles.alive.len())
            .filter(|&slot| vehicles.alive[slot])
            .map(|slot| vehicles.id[slot])
            .collect();
        assert_eq!(snapshot.ids.len(), sim.vehicle_count() as usize);
        assert_eq!(snapshot.ids, live);
        assert!(snapshot.heading.iter().all(|h| h.is_finite()));
    }
}
