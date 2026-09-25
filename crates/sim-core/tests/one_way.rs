use trapiks_sim_core::fixtures::one_way_pair;
use trapiks_sim_core::network::Network;

#[test]
fn movements_respect_one_way_streets() {
    let mut network = Network::from_map(&one_way_pair());
    network.build_all_junctions();
    for node in [0, 1] {
        let junction = network.junction(node).expect("built");
        assert!(!junction.movements.is_empty());
        for m in &junction.movements {
            assert!(network.link_lanes(m.from_link) > 0);
            assert!(network.link_lanes(m.to_link) > 0);
            assert_eq!(network.link_to(m.from_link), node);
            assert_eq!(network.link_from(m.to_link), node);
        }
    }
}

#[test]
fn link_lane_counts_match_the_map() {
    let network = Network::from_map(&one_way_pair());
    assert_eq!(network.link_lanes(0), 1);
    assert_eq!(network.link_lanes(1), 1);
    assert_eq!(network.link_lanes(6), 2);
    assert_eq!(network.link_lanes(7), 0);
    assert!(!network.is_link_active(7));
    let incoming: Vec<u32> = network.incoming(0).collect();
    let outgoing: Vec<u32> = network.outgoing(0).collect();
    assert_eq!(incoming, vec![0, 3, 6]);
    assert_eq!(outgoing, vec![1, 2, 8]);
}
