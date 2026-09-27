use trapiks_sim_core::fixtures::{GridCity, MapBuilder, RoadSpec, grid_city};
use trapiks_sim_core::map::{MapData, RoadClass};
use trapiks_sim_core::network::{LinkId, Network};
use trapiks_sim_core::routing::{
    RouteGraph, largest_component, largest_component_members, reachable_from, reaching,
};

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

struct Feeder {
    map: MapData,
    source: u32,
    sink: u32,
    spur: u32,
}

fn feeder() -> Feeder {
    let mut builder = MapBuilder::new();
    let a = builder.node(-300.0, 0.0);
    let b = builder.node(0.0, 0.0);
    let c = builder.node(300.0, 0.0);
    let d = builder.node(150.0, 250.0);
    let e = builder.node(150.0, 550.0);
    let f = builder.node(600.0, 0.0);
    let two_way = || RoadSpec::new(RoadClass::Primary, 1, 1);
    let one_way = || RoadSpec::new(RoadClass::Primary, 1, 0);
    let source = builder.road(a, b, one_way());
    builder.road(b, c, two_way());
    builder.road(c, d, two_way());
    builder.road(d, b, two_way());
    let sink = builder.road(d, e, one_way());
    let spur = builder.road(c, f, one_way());
    Feeder {
        map: builder.build(),
        source,
        sink,
        spur,
    }
}

#[test]
fn boundary_source_and_sink_reach_the_core() {
    let feeder = feeder();
    let network = Network::from_map(&feeder.map);
    let graph = RouteGraph::build(&network);
    let active = |link: LinkId| network.is_link_active(link);
    let core = largest_component_members(&graph, active);
    let can_leave = reaching(&graph, &core, active);
    let can_arrive = reachable_from(&graph, &core, active);
    let forward = |road: u32| network.departing_link(road, network.roads.from[road as usize]);
    let (source, sink, spur) = (
        forward(feeder.source),
        forward(feeder.sink),
        forward(feeder.spur),
    );
    assert!(!core[source as usize] && !core[sink as usize] && !core[spur as usize]);
    assert!(can_leave[source as usize] && !can_arrive[source as usize]);
    assert!(can_arrive[sink as usize] && !can_leave[sink as usize]);
    assert!(can_arrive[spur as usize] && !can_leave[spur as usize]);
    assert_eq!(core.iter().filter(|&&m| m).count(), 3);
}
