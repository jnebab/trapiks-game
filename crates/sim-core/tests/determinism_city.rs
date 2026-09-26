use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::stats::StatsSnapshot;

fn run() -> (u64, StatsSnapshot) {
    let map = grid_city(&GridCity {
        cols: 40,
        rows: 40,
        spacing: 150.0,
    });
    let config = SimConfig {
        seed: 17,
        mode: SimMode::City,
        vehicles_per_hour: 20_000.0,
        budget: None,
    };
    let mut sim = Sim::from_config(&map, &config);
    for _ in 0..3_000 {
        sim.step();
    }
    (sim.state_hash(), sim.stats())
}

#[test]
fn determinism_city() {
    let (hash, stats) = run();
    assert!(stats.spawned > 0, "{stats:?}");
    assert_eq!(run(), (hash, stats));
}
