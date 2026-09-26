use trapiks_sim_core::consts::LANDMARK_COUNT;
use trapiks_sim_core::fixtures::{
    GridCity, MapBuilder, RoadSpec, four_way, grid_city, one_way_pair,
};
use trapiks_sim_core::map::{MapData, RoadClass, TurnBan};
use trapiks_sim_core::network::{LinkId, Network};
use trapiks_sim_core::rng::Pcg32;
use trapiks_sim_core::routing::{
    Landmarks, LinkCosts, RouteContext, RouteGraph, Router, dijkstra_cost,
};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::SpawnError;

struct Routing {
    network: Network,
    graph: RouteGraph,
    costs: LinkCosts,
    landmarks: Landmarks,
}

impl Routing {
    fn new(map: &MapData) -> Self {
        let network = Network::from_map(map);
        let graph = RouteGraph::build(&network);
        let costs = LinkCosts::build(&network);
        let landmarks = Landmarks::build(&network, &graph, &costs);
        Routing {
            network,
            graph,
            costs,
            landmarks,
        }
    }

    fn route(&self, with_landmarks: bool, from: LinkId, to: LinkId) -> Option<Vec<LinkId>> {
        let ctx = RouteContext {
            graph: &self.graph,
            costs: &self.costs,
            landmarks: with_landmarks.then_some(&self.landmarks),
            network: &self.network,
        };
        let mut out = Vec::new();
        Router::new()
            .route_into(ctx, from, to, &mut out)
            .then_some(out)
    }

    fn cost(&self, route: &[LinkId]) -> f64 {
        route
            .windows(2)
            .map(|pair| {
                self.graph
                    .successors(pair[0])
                    .iter()
                    .find(|s| s.to == pair[1])
                    .map_or(f64::INFINITY, |succ| {
                        self.costs.travel_time(pair[1]) + f64::from(succ.penalty)
                    })
            })
            .sum()
    }

    fn random_active(&self, rng: &mut Pcg32) -> LinkId {
        loop {
            let link = rng.below(self.network.link_count() as u32);
            if self.network.is_link_active(link) {
                return link;
            }
        }
    }
}

fn grid(size: u32) -> MapData {
    grid_city(&GridCity {
        cols: size,
        rows: size,
        spacing: 150.0,
    })
}

#[test]
fn astar_optimal() {
    let routing = Routing::new(&grid(10));
    let mut rng = Pcg32::new(11, 3);
    for _ in 0..50 {
        let from = routing.random_active(&mut rng);
        let to = routing.random_active(&mut rng);
        let expected = dijkstra_cost(&routing.graph, &routing.costs, from, to);
        for with_landmarks in [true, false] {
            let route = routing.route(with_landmarks, from, to);
            let cost = route.map(|r| routing.cost(&r));
            match (expected, cost) {
                (Some(e), Some(c)) => assert!((e - c).abs() <= 1e-6 * e.max(1.0), "{e} {c}"),
                (None, None) => {}
                other => panic!("{from}->{to}: {other:?}"),
            }
        }
    }
}

#[test]
fn astar_respects_bans() {
    let mut map = four_way(2, 200.0);
    map.turn_bans.push(TurnBan {
        via_node: 0,
        from_road: 0,
        to_road: 1,
    });
    let routing = Routing::new(&map);
    let route = routing.route(true, 0, 3).expect("route");
    assert!(route.len() > 2, "{route:?}");
    assert!(route.windows(2).all(|pair| pair != [0, 3]));
}

#[test]
fn astar_respects_one_ways() {
    let routing = Routing::new(&one_way_pair());
    let mut network = Network::from_map(&one_way_pair());
    let links = routing.network.link_count() as LinkId;
    for from in 0..links {
        for to in 0..links {
            let Some(route) = routing.route(true, from, to) else {
                continue;
            };
            assert!(route.iter().all(|&l| network.is_link_active(l)));
            for pair in route.windows(2) {
                let node = network.link_to(pair[0]);
                let junction = network.ensure_junction(node);
                assert!(junction.movement_index(pair[0], pair[1]).is_some());
            }
        }
    }
}

fn two_components() -> MapData {
    let mut b = MapBuilder::new();
    let n = [(0.0, 0.0), (200.0, 0.0), (0.0, 500.0), (200.0, 500.0)].map(|(x, y)| b.node(x, y));
    let street = RoadSpec::new(RoadClass::Tertiary, 1, 1);
    b.road(n[0], n[1], street.clone());
    b.road(n[2], n[3], street);
    b.build()
}

#[test]
fn unreachable() {
    let map = two_components();
    let mut sim = Sim::new(&map, 1);
    assert!(sim.route(0, 1).is_some());
    assert_eq!(sim.route(0, 2), None);
    assert_eq!(sim.spawn_trip(0, 2), Err(SpawnError::NoRoute));
}

#[test]
fn ema_refresh() {
    let network = Network::from_map(&four_way(2, 200.0));
    let mut costs = LinkCosts::build(&network);
    let free = costs.free_speed[0];
    for _ in 0..10 {
        for _ in 0..5 {
            costs.accumulate(0, free / 2.0);
        }
        costs.refresh();
    }
    let half = free / 2.0;
    assert!((costs.ema_speed[0] - half).abs() <= 0.05 * half);
    for _ in 0..20 {
        costs.refresh();
    }
    assert!((costs.ema_speed[0] - free).abs() <= 0.05 * free);
}

#[test]
fn landmarks_admissible() {
    let routing = Routing::new(&grid(20));
    assert_eq!(routing.landmarks.count(), LANDMARK_COUNT);
    let mut rng = Pcg32::new(5, 9);
    for _ in 0..200 {
        let v = routing.random_active(&mut rng);
        let t = routing.random_active(&mut rng);
        let Some(cost) = dijkstra_cost(&routing.graph, &routing.costs, v, t) else {
            continue;
        };
        let target = routing.landmarks.target(&routing.network, t);
        let h = routing
            .landmarks
            .estimate(&routing.network, &routing.costs, v, &target);
        assert!(h <= cost + 1e-3, "{v}->{t}: h {h} cost {cost}");
    }
}
