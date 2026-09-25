use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::network::Network;

#[test]
fn rect_query_finds_roads() {
    let network = Network::from_map(&four_way(2, 200.0));
    let spatial = &network.spatial;
    assert_eq!(
        spatial.roads_in_rect(Vec2::new(-10.0, -10.0), Vec2::new(10.0, 10.0)),
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        spatial.roads_in_rect(Vec2::new(150.0, -5.0), Vec2::new(190.0, 5.0)),
        vec![1]
    );
}

#[test]
fn radius_query_finds_nodes() {
    let network = Network::from_map(&four_way(2, 200.0));
    let spatial = &network.spatial;
    assert_eq!(
        spatial.nodes_within(&network.nodes, Vec2::new(0.0, 0.0), 10.0),
        vec![0]
    );
    assert_eq!(
        spatial.nodes_within(&network.nodes, Vec2::new(0.0, 0.0), 200.0),
        vec![0, 1, 2, 3, 4]
    );
    assert_eq!(
        spatial.nodes_within(&network.nodes, Vec2::new(200.0, 0.0), 1.0),
        vec![2]
    );
    assert_eq!(
        spatial.roads_near(&network.roads, Vec2::new(0.0, -195.0), 10.0),
        vec![0]
    );
}
