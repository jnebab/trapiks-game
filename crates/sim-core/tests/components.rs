use trapiks_sim_core::fixtures::{GridCity, MapBuilder, RoadSpec, grid_city};
use trapiks_sim_core::map::{MapData, RoadClass};
use trapiks_sim_core::network::{LinkId, Network};
use trapiks_sim_core::routing::{RouteGraph, largest_component};

fn largest_and_active(map: &MapData) -> (usize, usize) {
    let network = Network::from_map(map);
    let graph = RouteGraph::build(&network);
    let active = (0..network.link_count() as LinkId)
        .filter(|&link| network.is_link_active(link))
        .count();
    let largest = largest_component(&graph, |link| network.is_link_active(link));
    (largest, active)
}

fn one_way_chain() -> MapData {
    let mut builder = MapBuilder::new();
    let a = builder.node(0.0, 0.0);
    let b = builder.node(200.0, 0.0);
    let c = builder.node(400.0, 0.0);
    builder.road(a, b, RoadSpec::new(RoadClass::Primary, 1, 0));
    builder.road(b, c, RoadSpec::new(RoadClass::Primary, 1, 0));
    builder.build()
}

fn reachable(graph: &RouteGraph, start: LinkId) -> Vec<bool> {
    let mut seen = vec![false; graph.link_count()];
    seen[start as usize] = true;
    let mut stack = vec![start];
    while let Some(link) = stack.pop() {
        for succ in graph.successors(link) {
            if !seen[succ.to as usize] {
                seen[succ.to as usize] = true;
                stack.push(succ.to);
            }
        }
    }
    seen
}

fn brute_largest(map: &MapData) -> usize {
    let network = Network::from_map(map);
    let graph = RouteGraph::build(&network);
    let active: Vec<LinkId> = (0..network.link_count() as LinkId)
        .filter(|&link| network.is_link_active(link))
        .collect();
    let reach: Vec<Vec<bool>> = (0..network.link_count() as LinkId)
        .map(|link| reachable(&graph, link))
        .collect();
    let mutual =
        |a: LinkId, b: LinkId| reach[a as usize][b as usize] && reach[b as usize][a as usize];
    active
        .iter()
        .map(|&a| active.iter().filter(|&&b| mutual(a, b)).count())
        .max()
        .unwrap_or(0)
}

#[test]
fn grid_matches_brute_force() {
    let map = grid_city(&GridCity {
        cols: 6,
        rows: 6,
        spacing: 150.0,
    });
    let (largest, active) = largest_and_active(&map);
    assert!(largest * 10 > active * 9);
    assert_eq!(largest, brute_largest(&map));
}

#[test]
fn one_way_chain_splits_into_singletons() {
    let (largest, active) = largest_and_active(&one_way_chain());
    assert_eq!(active, 2);
    assert_eq!(largest, 1);
}
