mod support;

use std::collections::BTreeMap;

use support::{apply_now, check_invariants, slot_of};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{EditCommand, EditError, EditOutcome, Outcome};
use trapiks_sim_core::fixtures::{GridCity, MapBuilder, RoadSpec, four_way, grid_city, t_junction};
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::{MapData, RoadClass};
use trapiks_sim_core::network::{LinkId, Network};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const NORTH_SOUTH: (LinkId, LinkId) = (0, 5);
const EAST_WEST: (LinkId, LinkId) = (2, 7);
const FLYOVER: EditCommand = EditCommand::BuildFlyover {
    node: 0,
    through: [2, 0],
};

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

fn touches(network: &Network, route: &[LinkId], node: u32) -> bool {
    route
        .windows(2)
        .any(|pair| network.link_to(pair[0]) == node)
}

#[derive(Default)]
struct Streams {
    north_south: Vec<u32>,
    speeds: BTreeMap<u32, (f64, u32)>,
}

impl Streams {
    fn spawn(&mut self, sim: &mut Sim) {
        if !sim.tick().is_multiple_of(40) {
            return;
        }
        if let Ok(id) = sim.spawn_trip(NORTH_SOUTH.0, NORTH_SOUTH.1) {
            self.north_south.push(id);
        }
        let _ = sim.spawn_trip(EAST_WEST.0, EAST_WEST.1);
    }

    fn step(&mut self, sim: &mut Sim) {
        self.spawn(sim);
        sim.step();
        for &id in &self.north_south {
            if let Some(slot) = slot_of(sim, id) {
                let entry = self.speeds.entry(id).or_default();
                entry.0 += sim.vehicles().v[slot as usize];
                entry.1 += 1;
            }
        }
    }

    fn finished_means(&self, sim: &Sim) -> Vec<f64> {
        self.speeds
            .iter()
            .filter(|&(&id, _)| slot_of(sim, id).is_none())
            .map(|(_, &(sum, count))| sum / f64::from(count))
            .collect()
    }
}

fn assert_fixed_ids(sim: &Sim, outcome: &EditOutcome, before: (usize, usize)) {
    let network = sim.network();
    let roads = &network.roads;
    let counts = (roads.count(), network.nodes.count());
    assert_eq!(counts, (before.0 + 2, before.1 + 3));
    assert_eq!(outcome.changed_roads, vec![0, 2, 4, 5]);
    assert_eq!(outcome.changed_nodes, vec![0, 5, 6, 7]);
    let ends: Vec<(u32, u32)> = (0..6).map(|r| (roads.from[r], roads.to[r])).collect();
    assert_eq!(ends, vec![(1, 6), (2, 0), (3, 7), (4, 0), (6, 5), (7, 5)]);
    assert_eq!((roads.layer[4], roads.layer[5]), (1, 1));
    assert_eq!(network.active_degree(0), 2);
}

fn route_touches(sim: &mut Sim, (from, to): (LinkId, LinkId), node: u32) -> Option<bool> {
    let route = sim.route(from, to)?;
    Some(touches(sim.network(), &route, node))
}

#[test]
fn flyover_bypasses_junction() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let before = (sim.network().roads.count(), sim.network().nodes.count());
    let outcome = ok(apply_now(&mut sim, FLYOVER));
    assert_fixed_ids(&sim, &outcome, before);
    assert_eq!(route_touches(&mut sim, NORTH_SOUTH, 5), Some(true));
    assert_eq!(route_touches(&mut sim, NORTH_SOUTH, 0), Some(false));
    assert_eq!(route_touches(&mut sim, EAST_WEST, 0), Some(true));
    let mut streams = Streams::default();
    for _ in 0..2_000 {
        streams.step(&mut sim);
        check_invariants(&sim);
    }
    let v0 = sim.network().roads.speed[0];
    let means = streams.finished_means(&sim);
    assert!(means.len() >= 10, "{}", means.len());
    assert!(
        means.iter().all(|&mean| mean >= 0.8 * v0),
        "{means:?} v0 {v0}"
    );
}

fn geometry(network: &Network, road: u32) -> Vec<Vec2> {
    network.roads.points(road).to_vec()
}

fn inside_flyover(sim: &Sim) -> u64 {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter(|&slot| match vehicles.place[slot as usize] {
            Place::Movement { node, .. } => (5..=7).contains(&node),
            Place::Link { .. } => false,
        })
        .count() as u64
}

fn max_route_link(sim: &Sim) -> LinkId {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .flat_map(|slot| vehicles.route(slot).iter().copied())
        .max()
        .unwrap_or_default()
}

fn assert_restored(
    sim: &Sim,
    hash: u64,
    counts: (usize, usize),
    originals: &[Vec<Vec2>],
) -> LinkId {
    let network = sim.network();
    assert_eq!(network.content_hash(), hash);
    assert_eq!((network.roads.count(), network.nodes.count()), counts);
    assert_eq!([0, 2].map(|road| geometry(network, road)), originals);
    assert_eq!(network.active_degree(0), 4);
    let limit = 2 * counts.0 as LinkId;
    assert!(max_route_link(sim) < limit);
    limit
}

fn flow_onto_flyover(sim: &mut Sim, streams: &mut Streams) -> bool {
    for _ in 0..300 {
        streams.step(sim);
    }
    ok(apply_now(sim, FLYOVER));
    for _ in 0..100 {
        streams.step(sim);
    }
    (0..400).any(|_| {
        streams.step(sim);
        inside_flyover(sim) > 0 && vehicle_on_road(sim, 4)
    })
}

fn undo_now(sim: &mut Sim) -> bool {
    let undo = sim.enqueue(EditCommand::Undo);
    sim.step();
    sim.take_results()
        .iter()
        .any(|r| r.seq == undo && matches!(r.outcome, Outcome::Ok(_)))
}

#[test]
fn flyover_undo() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let hash = sim.network().content_hash();
    let (roads, nodes) = (sim.network().roads.count(), sim.network().nodes.count());
    let originals = [0, 2].map(|road| geometry(sim.network(), road));
    let mut streams = Streams::default();
    assert!(flow_onto_flyover(&mut sim, &mut streams));
    let expected = inside_flyover(&sim);
    let stranded = sim.stranded();
    assert!(undo_now(&mut sim));
    assert_eq!(sim.stranded() - stranded, expected);
    let limit = assert_restored(&sim, hash, (roads, nodes), &originals);
    for _ in 0..200 {
        streams.step(&mut sim);
        check_invariants(&sim);
        assert!(max_route_link(&sim) < limit);
    }
    assert_eq!(sim.network_mut().ensure_junction(0).movements.len(), 12);
}

#[test]
fn flyover_errors() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let build = |node, through| EditCommand::BuildFlyover { node, through };
    assert_eq!(err(&mut sim, build(1, [0, 2])), EditError::NotAJunction);
    assert_eq!(err(&mut sim, build(9, [0, 2])), EditError::NodeNotFound);
    assert_eq!(err(&mut sim, build(0, [0, 0])), EditError::RoadNotIncident);
    assert_eq!(err(&mut sim, build(0, [0, 9])), EditError::RoadNotIncident);
    assert_eq!(
        err(&mut sim, build(0, [0, 1])),
        EditError::FlyoverNotStraight
    );
    let mut short = Sim::new(&four_way(2, 20.0), 1);
    assert_eq!(
        err(&mut short, build(0, [0, 2])),
        EditError::FlyoverTooShort
    );
    let mut tee = Sim::new(&t_junction(), 1);
    ok(apply_now(&mut tee, EditCommand::DeleteRoad { road: 2 }));
    assert_eq!(err(&mut tee, build(0, [0, 1])), EditError::NotAJunction);
}

fn vehicle_on_road(sim: &Sim, road: u32) -> bool {
    let vehicles = sim.vehicles();
    vehicles.live_slots().any(|slot| {
        matches!(vehicles.place[slot as usize], Place::Link { link, .. } if link / 2 == road)
    })
}

#[test]
fn flyover_then_edit_undo_twice() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let hash = sim.network().content_hash();
    let originals = [0, 2].map(|road| geometry(sim.network(), road));
    ok(apply_now(&mut sim, FLYOVER));
    let mut streams = Streams::default();
    let found = (0..600).any(|_| {
        streams.step(&mut sim);
        vehicle_on_road(&sim, 4)
    });
    assert!(found);
    let lanes = EditCommand::SetLanes {
        road: 4,
        forward: 1,
        backward: 2,
    };
    ok(apply_now(&mut sim, lanes));
    check_invariants(&sim);
    ok(apply_now(&mut sim, EditCommand::Undo));
    check_invariants(&sim);
    ok(apply_now(&mut sim, EditCommand::Undo));
    let network = sim.network();
    assert_eq!(network.content_hash(), hash);
    assert_eq!(geometry(network, 0), originals[0]);
    assert_eq!(geometry(network, 2), originals[1]);
    for _ in 0..200 {
        streams.step(&mut sim);
        check_invariants(&sim);
    }
    assert_eq!(sim.undo_depth(), 0);
}

fn region_sim() -> Sim {
    let map = grid_city(&GridCity {
        cols: 10,
        rows: 10,
        spacing: 150.0,
    });
    let config = SimConfig {
        seed: 3,
        mode: SimMode::Region {
            center_x: 750.0,
            center_y: 750.0,
            radius: 400.0,
        },
        vehicles_per_hour: 3_000.0,
        budget: None,
    };
    Sim::from_config(&map, &config)
}

fn opposite_pair(network: &Network, node: u32) -> Option<[u32; 2]> {
    let roads = &network.nodes.roads[node as usize];
    let direction = |road: u32| {
        let link = network.departing_link(road, node);
        network.centre_pose(link, 10.0).1
    };
    roads.iter().enumerate().find_map(|(i, &a)| {
        roads[i + 1..]
            .iter()
            .find(|&&b| direction(a).dot(direction(b)) < -0.9)
            .map(|&b| [a, b])
    })
}

fn find_node(
    network: &Network,
    wanted: impl Fn(&Network, [u32; 2]) -> bool,
) -> Option<(u32, [u32; 2])> {
    (0..network.nodes.count() as u32)
        .filter(|&node| network.active_degree(node) >= 3)
        .find_map(|node| {
            let pair = opposite_pair(network, node)?;
            wanted(network, pair).then_some((node, pair))
        })
}

fn hooked_cross() -> MapData {
    let mut b = MapBuilder::new();
    let centre = b.node(0.0, 0.0);
    let arms =
        [(0.0, -200.0), (200.0, 0.0), (0.0, 200.0), (-200.0, 0.0)].map(|(x, y)| b.node(x, y));
    let hooks = [vec![], vec![(20.0, -150.0)], vec![], vec![(-20.0, -150.0)]];
    for (node, via) in arms.into_iter().zip(hooks) {
        let spec = RoadSpec::new(RoadClass::Primary, 1, 1).via(via);
        b.road(node, centre, spec);
    }
    b.build()
}

#[test]
fn flyover_region() {
    let config = SimConfig {
        seed: 1,
        mode: SimMode::Region {
            center_x: 0.0,
            center_y: -150.0,
            radius: 60.0,
        },
        vehicles_per_hour: 0.0,
        budget: None,
    };
    let mut hooked = Sim::from_config(&hooked_cross(), &config);
    assert_eq!(hooked.network().active_degree(0), 3);
    let command = EditCommand::BuildFlyover {
        node: 0,
        through: [0, 2],
    };
    assert_eq!(err(&mut hooked, command), EditError::OutsideRegion);
    let mut sim = region_sim();
    for _ in 0..300 {
        sim.step();
    }
    let (node, inside) = find_node(sim.network(), |network, pair| {
        pair.iter().all(|&road| network.is_road_active(road))
    })
    .expect("node");
    let roads = sim.network().roads.count() as u32;
    ok(apply_now(
        &mut sim,
        EditCommand::BuildFlyover {
            node,
            through: inside,
        },
    ));
    assert!(sim.network().is_road_active(roads));
    assert!(sim.network().is_road_active(roads + 1));
    for _ in 0..500 {
        sim.step();
        check_invariants(&sim);
    }
    assert!(sim.vehicle_count() > 0);
}
