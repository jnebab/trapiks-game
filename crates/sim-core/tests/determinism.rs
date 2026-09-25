use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::network::{Junction, Network};

fn junctions() -> Vec<Junction> {
    let spec = GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    };
    let mut network = Network::from_map(&grid_city(&spec));
    network.build_all_junctions();
    (0..network.nodes.count() as u32)
        .filter_map(|node| network.junction(node).cloned())
        .collect()
}

#[test]
fn junction_build_is_deterministic() {
    let first = junctions();
    assert!(first.iter().any(|j| !j.conflicts.iter().all(Vec::is_empty)));
    assert_eq!(first, junctions());
}
