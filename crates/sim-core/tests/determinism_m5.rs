use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::Network;
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::SpawnError;

const FOUR_WAY_HASH_5000: u64 = 0x74ed_b2c4_8572_d418;

fn run() -> u64 {
    let map = four_way(2, 200.0);
    let mut network = Network::from_map(&map);
    let movements: Vec<(u32, u32)> = network
        .ensure_junction(0)
        .movements
        .iter()
        .map(|m| (m.from_link, m.to_link))
        .collect();
    let mut sim = Sim::new(&map, 7);
    let mut test_rng = Pcg32::new(99, 1);
    for tick in 0..5_000 {
        if tick % 10 == 0 {
            let (from, to) = movements[test_rng.below(12) as usize];
            match sim.spawn(&[from, to]) {
                Ok(_) | Err(SpawnError::Blocked) => {}
                Err(error) => panic!("{error:?}"),
            }
        }
        sim.step();
    }
    sim.state_hash()
}

#[test]
fn four_way_hash_is_reproducible() {
    let first = run();
    assert_eq!(first, run());
    assert_eq!(first, FOUR_WAY_HASH_5000, "{first:#018x}");
}
