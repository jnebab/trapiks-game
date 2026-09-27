use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::consts::MAX_VEHICLES;
use trapiks_sim_core::fixtures::{GridCity, corridor, grid_city};
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

fn grid_sim(cols: u32, seed: u64) -> Sim {
    let map = grid_city(&GridCity {
        cols,
        rows: cols,
        spacing: 150.0,
    });
    let config = SimConfig {
        seed,
        mode: SimMode::City,
        vehicles_per_hour: 8_000.0,
        budget: None,
    };
    Sim::from_config(&map, &config)
}

fn preallocated() -> Snapshot {
    Snapshot {
        ids: Vec::with_capacity(MAX_VEHICLES),
        x: Vec::with_capacity(MAX_VEHICLES),
        y: Vec::with_capacity(MAX_VEHICLES),
        heading: Vec::with_capacity(MAX_VEHICLES),
        style: Vec::with_capacity(MAX_VEHICLES),
        layer: Vec::with_capacity(MAX_VEHICLES),
    }
}

fn layout(s: &Snapshot) -> [(usize, usize); 6] {
    [
        (s.ids.as_ptr() as usize, s.ids.capacity()),
        (s.x.as_ptr() as usize, s.x.capacity()),
        (s.y.as_ptr() as usize, s.y.capacity()),
        (s.heading.as_ptr() as usize, s.heading.capacity()),
        (s.style.as_ptr() as usize, s.style.capacity()),
        (s.layer.as_ptr() as usize, s.layer.capacity()),
    ]
}

#[test]
fn snapshot_buffers_never_reallocate() {
    let mut sim = grid_sim(20, 5);
    let mut snapshot = preallocated();
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

#[test]
fn snapshot_reports_the_layer_under_each_vehicle() {
    let mut sim = grid_sim(12, 3);
    let mut snapshot = Snapshot::default();
    let mut elevated = 0;
    for _ in 0..1_500 {
        sim.step();
        sim.fill_snapshot(&mut snapshot);
        assert_eq!(snapshot.layer.len(), snapshot.ids.len());
        assert!(snapshot.layer.iter().all(|&layer| layer == 0 || layer == 1));
        elevated += snapshot.layer.iter().filter(|&&layer| layer == 1).count();
    }
    assert!(elevated > 0);
}
