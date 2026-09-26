use trapiks_sim_core::consts::LANDMARK_COUNT;
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::network::{Direction, Network, Region, direction_of, link_id};
use trapiks_sim_core::routing::{Landmarks, LinkCosts, RouteGraph};

fn regional() -> Network {
    let spec = GridCity {
        cols: 10,
        rows: 10,
        spacing: 150.0,
    };
    let mut network = Network::from_map(&grid_city(&spec));
    network.set_region(Some(Region {
        center: Vec2::new(750.0, 750.0),
        radius: 400.0,
    }));
    network
}

#[test]
fn region_covers_part_of_the_city() {
    let network = regional();
    let active = (0..network.roads.count() as u32)
        .filter(|&r| network.is_road_active(r))
        .count();
    assert!(active > 0 && active < network.roads.count());
    assert_eq!(network.version(), 1);
}

#[test]
fn boundary_nodes_straddle_the_region() {
    let network = regional();
    let mask = network.region().expect("mask");
    assert!(!mask.boundary.is_empty());
    for &node in &mask.boundary {
        let roads = &network.nodes.roads[node as usize];
        assert!(roads.iter().any(|&r| network.is_road_active(r)));
        assert!(
            roads
                .iter()
                .any(|&r| !network.is_road_active(r) && network.roads.is_live(r))
        );
    }
    assert!(!mask.sources.is_empty());
    assert!(!mask.sinks.is_empty());
    assert!(mask.sources.iter().all(|&l| network.is_link_active(l)));
}

#[test]
fn inactive_roads_have_inactive_links() {
    let network = regional();
    let inactive = (0..network.roads.count() as u32)
        .find(|&r| !network.is_road_active(r))
        .expect("inactive road");
    for dir in [Direction::Forward, Direction::Backward] {
        let link = link_id(inactive, dir);
        assert_eq!(direction_of(link), dir);
        assert!(!network.is_link_active(link));
    }
}

#[test]
fn region_gets_full_landmark_set() {
    let network = regional();
    let graph = RouteGraph::build(&network);
    let costs = LinkCosts::build(&network);
    let landmarks = Landmarks::build(&network, &graph, &costs);
    assert_eq!(landmarks.count(), LANDMARK_COUNT);
}
