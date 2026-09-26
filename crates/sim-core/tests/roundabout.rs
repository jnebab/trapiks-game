mod support;

use std::collections::BTreeMap;

use support::{apply_now, check_invariants};
use trapiks_sim_core::edit::{EditCommand, EditError, EditOutcome, Outcome};
use trapiks_sim_core::fixtures::{MapBuilder, RoadSpec, four_way};
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::{MapData, RoadClass};
use trapiks_sim_core::network::{LinkId, Movement, Network, TurnKind, movement_is_minor};
use trapiks_sim_core::sim::Sim;

const BUILD: EditCommand = EditCommand::BuildRoundabout {
    node: 0,
    radius_m: 18,
};
const ARRIVING: [LinkId; 4] = [0, 2, 4, 6];
const DEPARTING: [LinkId; 4] = [1, 3, 5, 7];
const SPAWN_EVERY: u64 = 60;
const MAX_WAIT_TICKS: u32 = 3_000;
const STOPPED: f64 = 0.1;

fn ok(outcome: Outcome) -> EditOutcome {
    match outcome {
        Outcome::Ok(outcome) => outcome,
        Outcome::Err(error) => panic!("edit failed: {error:?}"),
    }
}

fn err(sim: &mut Sim, command: EditCommand) -> EditError {
    match apply_now(sim, command) {
        Outcome::Err(error) => error,
        Outcome::Ok(outcome) => panic!("edit accepted: {outcome:?}"),
    }
}

fn build_sim() -> Sim {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    ok(apply_now(&mut sim, BUILD));
    sim
}

#[test]
fn roundabout_ids_follow_append_order() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let outcome = ok(apply_now(&mut sim, BUILD));
    let network = sim.network();
    let roads = &network.roads;
    assert_eq!((roads.count(), network.nodes.count()), (8, 9));
    assert_eq!(outcome.changed_roads, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(outcome.changed_nodes, vec![0, 5, 6, 7, 8]);
    let ends: Vec<(u32, u32)> = (0..8).map(|r| (roads.from[r], roads.to[r])).collect();
    assert_eq!(
        ends,
        vec![
            (1, 5),
            (2, 6),
            (3, 7),
            (4, 8),
            (5, 8),
            (8, 7),
            (7, 6),
            (6, 5)
        ]
    );
    assert_eq!(network.active_degree(0), 0);
    for node in 5..9 {
        assert_eq!(network.active_degree(node), 3);
    }
}

#[test]
fn roundabout_ring_geometry() {
    let sim = build_sim();
    let network = sim.network();
    let centre = Vec2::default();
    for road in 4..8u32 {
        assert!(network.roads.is_roundabout(road));
        assert_eq!(network.roads.lanes_backward[road as usize], 0);
        assert_eq!(network.roads.lanes_forward[road as usize], 2);
        let points = network.roads.points(road);
        for point in points {
            assert!((point.distance(centre) - 18.0).abs() <= 0.01, "{point:?}");
        }
        let first = points[0] - centre;
        let last = points[points.len() - 1] - centre;
        assert!(libm::atan2(first.cross(last), first.dot(last)).abs() < std::f64::consts::PI);
    }
    let ring: Vec<Vec2> = [5, 8, 7, 6].map(|n| network.nodes.pos[n]).to_vec();
    let shoelace: f64 = (0..4)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % 4]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    assert!(shoelace < 0.0, "{shoelace}");
}

fn check_ring_movement(network: &Network, node: u32, movement: &Movement) {
    let from_ring = network.roads.is_roundabout(movement.from_link / 2);
    let to_ring = network.roads.is_roundabout(movement.to_link / 2);
    let (kind, lanes) = if from_ring && to_ring {
        (TurnKind::Through, (0, 1))
    } else {
        (TurnKind::Right, (0, 0))
    };
    assert_eq!(movement.kind, kind);
    if from_ring {
        assert_eq!(movement.from_lanes, lanes);
        assert_eq!(movement.rank.0, 15);
        return;
    }
    assert_eq!(movement.primary_lanes.1, 0);
    assert!(movement_is_minor(network, node, movement));
}

#[test]
fn roundabout_turn_kinds_and_priority() {
    let mut sim = build_sim();
    let network = sim.network_mut();
    for node in 5..9u32 {
        let junction = network.ensure_junction(node).clone();
        for movement in &junction.movements {
            check_ring_movement(network, node, movement);
        }
    }
}

#[test]
fn roundabout_routes_use_ring_links() {
    let mut sim = build_sim();
    let ring_links = |sim: &mut Sim, to: LinkId| {
        let route = sim.route(4, to).expect("route");
        route.iter().filter(|&&link| link >= 8).count()
    };
    assert_eq!(ring_links(&mut sim, 1), 2);
    assert_eq!(ring_links(&mut sim, 3), 1);
    assert_eq!(ring_links(&mut sim, 7), 3);
}

#[derive(Default)]
struct Traffic {
    stopped: BTreeMap<u32, u32>,
    longest: u32,
}

impl Traffic {
    fn step(&mut self, sim: &mut Sim) {
        let tick = sim.tick();
        if tick.is_multiple_of(SPAWN_EVERY) {
            let turn = (tick / SPAWN_EVERY) as usize;
            for (arm, &from) in ARRIVING.iter().enumerate() {
                let to = DEPARTING[(arm + 1 + turn % 3) % 4];
                let _ = sim.spawn_trip(from, to);
            }
        }
        sim.step();
        self.track(sim);
    }

    fn track(&mut self, sim: &Sim) {
        let vehicles = sim.vehicles();
        let mut next = BTreeMap::new();
        for slot in vehicles.live_slots() {
            let index = slot as usize;
            let id = vehicles.id[index];
            let wait = if vehicles.v[index] < STOPPED {
                self.stopped.get(&id).copied().unwrap_or_default() + 1
            } else {
                0
            };
            assert!(wait <= MAX_WAIT_TICKS, "vehicle {id} waited {wait} ticks");
            self.longest = self.longest.max(wait);
            next.insert(id, wait);
        }
        self.stopped = next;
    }
}

#[test]
fn roundabout_flows() {
    let mut sim = build_sim();
    let mut traffic = Traffic::default();
    for _ in 0..3_000 {
        traffic.step(&mut sim);
        check_invariants(&sim);
    }
    let stats = sim.stats();
    eprintln!(
        "arrivals {} stranded {} active {} longest stop {} ticks",
        stats.arrivals, stats.stranded, stats.active, traffic.longest
    );
    assert!(stats.arrivals >= 100, "{}", stats.arrivals);
    assert_eq!(stats.stranded, 0);
}

fn geometry(network: &Network, road: u32) -> Vec<Vec2> {
    network.roads.points(road).to_vec()
}

fn max_route_link(sim: &Sim) -> LinkId {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .flat_map(|slot| vehicles.route(slot).iter().copied())
        .max()
        .unwrap_or_default()
}

#[test]
fn roundabout_undo() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let hash = sim.network().content_hash();
    let counts = (sim.network().roads.count(), sim.network().nodes.count());
    let originals: Vec<Vec<Vec2>> = (0..4).map(|r| geometry(sim.network(), r)).collect();
    let mut traffic = Traffic::default();
    for _ in 0..300 {
        traffic.step(&mut sim);
    }
    ok(apply_now(&mut sim, BUILD));
    for _ in 0..100 {
        traffic.step(&mut sim);
        check_invariants(&sim);
    }
    ok(apply_now(&mut sim, EditCommand::Undo));
    let network = sim.network();
    assert_eq!(network.content_hash(), hash);
    assert_eq!((network.roads.count(), network.nodes.count()), counts);
    let restored: Vec<Vec<Vec2>> = (0..4).map(|r| geometry(network, r)).collect();
    assert_eq!(restored, originals);
    let limit = 2 * counts.0 as LinkId;
    for _ in 0..200 {
        traffic.step(&mut sim);
        check_invariants(&sim);
        assert!(max_route_link(&sim) < limit);
    }
    assert_eq!(sim.network_mut().ensure_junction(0).movements.len(), 12);
}

fn star(lengths: [f32; 4], layers: [i8; 4], angles_deg: [f64; 4]) -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    for i in 0..4 {
        let angle = angles_deg[i].to_radians();
        let (x, y) = (libm::cos(angle) as f32, libm::sin(angle) as f32);
        let arm = b.node(x * lengths[i], y * lengths[i]);
        let spec = RoadSpec::new(RoadClass::Primary, 1, 1).layer(layers[i]);
        b.road(arm, centre, spec);
    }
    b.build()
}

const SQUARE: [f64; 4] = [0.0, 90.0, 180.0, 270.0];

fn build(node: u32, radius_m: u8) -> EditCommand {
    EditCommand::BuildRoundabout { node, radius_m }
}

#[test]
fn roundabout_errors_on_four_way() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    assert_eq!(err(&mut sim, build(9, 18)), EditError::NodeNotFound);
    assert_eq!(err(&mut sim, build(1, 18)), EditError::NotAJunction);
    assert_eq!(err(&mut sim, build(0, 15)), EditError::InvalidRadius);
    assert_eq!(err(&mut sim, build(0, 41)), EditError::InvalidRadius);
    let mut built = build_sim();
    assert_eq!(err(&mut built, build(5, 18)), EditError::AlreadyRoundabout);
    let lanes = EditCommand::SetLanes {
        road: 4,
        forward: 1,
        backward: 1,
    };
    assert_eq!(err(&mut built, lanes), EditError::InvalidLanes);
    let mut line = Sim::new(&line_map(), 1);
    assert_eq!(line.network().active_degree(1), 2);
    assert_eq!(err(&mut line, build(1, 18)), EditError::NotAJunction);
}

#[test]
fn roundabout_errors_on_arm_shape() {
    let mut short = Sim::new(&star([200.0, 200.0, 40.0, 200.0], [0; 4], SQUARE), 1);
    assert_eq!(err(&mut short, build(0, 28)), EditError::RoundaboutTooLarge);
    let mut levels = Sim::new(&star([200.0; 4], [0, 0, 1, 0], SQUARE), 1);
    assert_eq!(err(&mut levels, build(0, 18)), EditError::LayerMismatch);
    let tight = [0.0, 10.0, 180.0, 270.0];
    let mut close = Sim::new(&star([200.0; 4], [0; 4], tight), 1);
    assert_eq!(err(&mut close, build(0, 18)), EditError::RoundaboutTooTight);
}

fn line_map() -> MapData {
    let mut b = MapBuilder::new();
    let nodes = [0.0, 100.0, 200.0].map(|x| b.node(x, 0.0));
    let spec = RoadSpec::new(RoadClass::Primary, 1, 1);
    b.road(nodes[0], nodes[1], spec.clone());
    b.road(nodes[1], nodes[2], spec);
    b.build()
}
