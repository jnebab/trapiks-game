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

#[test]
fn snapshot_buffers_never_reallocate() {
    use trapiks_sim_core::config::{SimConfig, SimMode};
    use trapiks_sim_core::consts::MAX_VEHICLES;
    use trapiks_sim_core::fixtures::{GridCity, grid_city};
    let map = grid_city(&GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    });
    let config = SimConfig {
        seed: 5,
        mode: SimMode::City,
        vehicles_per_hour: 8_000.0,
        budget: None,
    };
    let mut sim = Sim::from_config(&map, &config);
    let mut snapshot = Snapshot {
        ids: Vec::with_capacity(MAX_VEHICLES),
        x: Vec::with_capacity(MAX_VEHICLES),
        y: Vec::with_capacity(MAX_VEHICLES),
        heading: Vec::with_capacity(MAX_VEHICLES),
        style: Vec::with_capacity(MAX_VEHICLES),
    };
    let layout = |s: &Snapshot| {
        [
            (s.ids.as_ptr() as usize, s.ids.capacity()),
            (s.x.as_ptr() as usize, s.x.capacity()),
            (s.y.as_ptr() as usize, s.y.capacity()),
            (s.heading.as_ptr() as usize, s.heading.capacity()),
            (s.style.as_ptr() as usize, s.style.capacity()),
        ]
    };
    let before = layout(&snapshot);
    let mut filled = 0;
    for _ in 0..1_000 {
        sim.step();
        sim.fill_snapshot(&mut snapshot);
        filled += usize::from(!snapshot.ids.is_empty());
    }
    assert!(filled > 0);
    assert_eq!(layout(&snapshot), before);
}
