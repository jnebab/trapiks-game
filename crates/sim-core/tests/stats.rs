use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::fixtures::{corridor, four_way};
use trapiks_sim_core::network::{SignalState, road_of};
use trapiks_sim_core::sim::Sim;

#[test]
fn stats_low_demand() {
    let config = SimConfig {
        seed: 5,
        mode: SimMode::City,
        vehicles_per_hour: 60.0,
    };
    let mut sim = Sim::from_config(&corridor(), &config);
    sim.set_trip_band_for_test((100.0, 2_000.0));
    for _ in 0..6_000 {
        sim.step();
    }
    let stats = sim.stats();
    assert!(stats.arrivals > 0, "{stats:?}");
    assert!(stats.mean_delay_s < 10.0, "{stats:?}");
    assert_eq!(
        stats.spawned - u64::from(stats.active) - stats.stranded,
        stats.arrivals
    );
    assert!(stats.stopped_share < 0.05, "{stats:?}");
}

#[test]
fn reset_stats_keeps_hash() {
    let config = SimConfig {
        seed: 5,
        mode: SimMode::City,
        vehicles_per_hour: 600.0,
    };
    let mut sim = Sim::from_config(&corridor(), &config);
    sim.set_trip_band_for_test((100.0, 2_000.0));
    for _ in 0..1_000 {
        sim.step();
    }
    let before = sim.state_hash();
    sim.reset_stats();
    assert_eq!(sim.state_hash(), before);
    let stats = sim.stats();
    assert_eq!((stats.created, stats.arrivals), (0, 0));
    assert_eq!(stats.throughput_per_hour, 0.0);
}

#[test]
fn road_speed_ratio_empty() {
    let sim = Sim::new(&four_way(2, 200.0), 1);
    let mut ratio = Vec::new();
    sim.road_speed_ratio(&mut ratio);
    assert_eq!(ratio.len(), sim.network().roads.count());
    assert!(ratio.iter().all(|&r| r == 1.0));
}

#[test]
fn road_speed_ratio_red_queue() {
    let map = four_way(2, 200.0);
    let mut sim = Sim::new(&map, 1);
    let network = sim.network();
    let red = network
        .incoming(0)
        .find(|&link| network.signal_state(link, 0) == Some(SignalState::Red))
        .expect("red approach");
    let exit = network
        .outgoing(0)
        .find(|&l| road_of(l) != road_of(red))
        .expect("exit");
    for tick in 0..320 {
        if tick < 120 {
            let _ = sim.spawn(&[red, exit]);
        }
        sim.step();
    }
    assert_eq!(
        sim.network().signal_state(red, sim.tick()),
        Some(SignalState::Red)
    );
    let mut ratio = Vec::new();
    sim.road_speed_ratio(&mut ratio);
    let value = ratio[road_of(red) as usize];
    assert!(value < 0.2, "{value}");
}
