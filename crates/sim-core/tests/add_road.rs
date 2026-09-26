mod support;

use support::{Must, apply_now, check_invariants, grid_10, slot_of};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{
    EditCommand, EditError, EditOutcome, Endpoint, Outcome, curve_points,
};
use trapiks_sim_core::geom::{QuadraticBezier, Vec2};
use trapiks_sim_core::network::{LinkId, Network, Region, road_of};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const ROWS: u32 = 11;
const ROW_ROADS: u32 = 110;
const SPLIT_ROAD: u32 = 32;
const TOLERANCE_M: f64 = 3.0;

fn node(i: u32, j: u32) -> u32 {
    i * ROWS + j
}

fn row_road(i: u32, j: u32) -> u32 {
    ROW_ROADS + j * 10 + i
}

fn at_node(i: u32, j: u32) -> Endpoint {
    Endpoint::Node { node: node(i, j) }
}

fn add(from: Endpoint, to: Endpoint, via: Option<[f64; 2]>, layer: i8) -> EditCommand {
    EditCommand::AddRoad {
        from,
        to,
        via,
        lanes_forward: 1,
        lanes_backward: 1,
        layer,
    }
}

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

fn city(vehicles_per_hour: f64) -> Sim {
    let config = SimConfig {
        seed: 3,
        mode: SimMode::City,
        vehicles_per_hour,
        budget: None,
    };
    Sim::from_config(&grid_10(), &config)
}

fn counts(sim: &Sim) -> (usize, usize) {
    (sim.network().roads.count(), sim.network().nodes.count())
}

fn route_is_connected(network: &Network, route: &[LinkId], cursor: usize) -> bool {
    route[cursor..].windows(2).all(|pair| {
        let node = network.link_to(pair[0]);
        network.link_from(pair[1]) == node
            && network
                .turns(node)
                .iter()
                .any(|&(from, to, _)| from == pair[0] && to == pair[1])
    })
}

fn assert_routes_connected(sim: &Sim) {
    let vehicles = sim.vehicles();
    for slot in vehicles.live_slots() {
        let route = vehicles.route(slot);
        assert!(
            route_is_connected(sim.network(), route, vehicles.cursor(slot)),
            "slot {slot} route {route:?} cursor {}",
            vehicles.cursor(slot)
        );
    }
}

fn step_checked(sim: &mut Sim, ticks: u32) {
    for _ in 0..ticks {
        sim.step();
        check_invariants(sim);
    }
}

#[test]
fn add_road_straight() {
    let mut sim = city(3_000.0);
    step_checked(&mut sim, 200);
    let before = counts(&sim);
    let outcome = ok(apply_now(
        &mut sim,
        add(at_node(2, 2), at_node(3, 3), None, 0),
    ));
    let new_road = before.0 as u32;
    assert_eq!(counts(&sim), (before.0 + 1, before.1));
    assert_eq!(
        outcome.cost,
        200 + (4.0 * 150.0 * 2f64.sqrt()).round() as i64
    );
    assert!(outcome.changed_roads.contains(&new_road));
    let from = row_road(1, 2) * 2;
    let to = row_road(3, 3) * 2;
    let route = sim.route(from, to).must("route");
    assert!(
        route.iter().any(|&link| road_of(link) == new_road),
        "{route:?}"
    );
    step_checked(&mut sim, 2_000);
}

struct Seen {
    id: u32,
    link: LinkId,
    s: f64,
}

fn on_links(sim: &Sim, links: &[LinkId]) -> Vec<Seen> {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter_map(|slot| match vehicles.place[slot as usize] {
            Place::Link { link, .. } if links.contains(&link) => Some(Seen {
                id: vehicles.id[slot as usize],
                link,
                s: vehicles.s[slot as usize],
            }),
            _ => None,
        })
        .collect()
}

fn in_movement_at(sim: &Sim, at: u32) -> usize {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter(|&slot| matches!(vehicles.place[slot as usize], Place::Movement { node, .. } if node == at))
        .count()
}

fn feed_column_three(sim: &mut Sim) {
    if sim.tick().is_multiple_of(15) {
        let _ = sim.spawn(&[66, 64, 62]);
    }
}

fn fill_split_road(sim: &mut Sim) {
    for _ in 0..3_000 {
        let seen = on_links(sim, &[SPLIT_ROAD * 2]);
        let before = seen.iter().any(|v| v.s > 10.0 && v.s < 70.0);
        let after = seen.iter().any(|v| v.s > 80.0 && v.s < 140.0);
        if before && after && in_movement_at(sim, node(3, 2)) > 0 {
            return;
        }
        feed_column_three(sim);
        sim.step();
    }
    panic!("split road never filled");
}

fn expected_place(seen: &Seen, new_road: u32) -> (LinkId, f64) {
    let r2 = new_road - 1;
    if seen.s >= 75.0 {
        (r2 * 2, seen.s - 75.0)
    } else {
        (seen.link, seen.s)
    }
}

fn assert_moved(sim: &Sim, seen: &[Seen], new_road: u32) {
    let clear_of_cut = seen
        .iter()
        .filter(|v| (v.s - 75.0).abs() > 5.0 && v.s < 140.0);
    for vehicle in clear_of_cut {
        let slot = slot_of(sim, vehicle.id).must("moved vehicle");
        let (link, s) = expected_place(vehicle, new_road);
        let Place::Link { link: now, .. } = sim.vehicles().place[slot as usize] else {
            panic!("vehicle {} left the link", vehicle.id);
        };
        let s_now = sim.vehicles().s[slot as usize];
        assert_eq!(now, link, "vehicle {}", vehicle.id);
        assert!(
            s_now >= s - 1e-9 && s_now <= s + TOLERANCE_M,
            "{s_now} vs {s}"
        );
    }
}

fn on_split_road(at_m: f64) -> Endpoint {
    Endpoint::OnRoad {
        road: SPLIT_ROAD,
        at_m,
    }
}

#[test]
fn add_road_on_road() {
    let mut sim = city(1_500.0);
    fill_split_road(&mut sim);
    let before = counts(&sim);
    let stranded = sim.stranded();
    let seen = on_links(&sim, &[SPLIT_ROAD * 2]);
    let command = add(at_node(2, 2), on_split_road(75.0), None, 0);
    ok(apply_now(&mut sim, command));
    assert_eq!(counts(&sim), (before.0 + 2, before.1 + 1));
    assert_moved(&sim, &seen, before.0 as u32 + 1);
    assert_routes_connected(&sim);
    assert_eq!(sim.stranded(), stranded);
    for _ in 0..1_000 {
        feed_column_three(&mut sim);
        sim.step();
        check_invariants(&sim);
    }
    assert_eq!(sim.stranded(), 0);
}

#[test]
fn add_road_crossing() {
    let mut sim = city(0.0);
    let command = |layer| add(at_node(2, 2), at_node(4, 3), None, layer);
    assert_eq!(err(&mut sim, command(0)), EditError::CrossesRoad);
    let new_road = sim.network().roads.count() as u32;
    let outcome = ok(apply_now(&mut sim, command(1)));
    assert_eq!(
        outcome.cost,
        ((200.0 + 4.0 * 5f64.sqrt() * 150.0) * 2.0).round() as i64
    );
    let from = row_road(1, 2) * 2;
    let to = row_road(4, 3) * 2;
    let id = sim.spawn_trip(from, to).must("spawn");
    let mut used = false;
    for _ in 0..1_500 {
        sim.step();
        let Some(slot) = slot_of(&sim, id) else {
            break;
        };
        used |= matches!(sim.vehicles().place[slot as usize], Place::Link { link, .. } if road_of(link) == new_road);
    }
    assert!(used);
}

fn split_geometry(network: &Network, road: u32) -> (Vec<Vec2>, f64, u32, u32) {
    let index = road as usize;
    (
        network.roads.points(road).to_vec(),
        network.roads.length[index],
        network.roads.from[index],
        network.roads.to[index],
    )
}

fn feed_rows(sim: &mut Sim) {
    if sim.tick().is_multiple_of(20) {
        let _ = sim.spawn(&[row_road(1, 2) * 2, row_road(2, 2) * 2, row_road(3, 2) * 2]);
        let _ = sim.spawn(&[
            row_road(3, 3) * 2 + 1,
            row_road(2, 3) * 2 + 1,
            row_road(1, 3) * 2 + 1,
        ]);
    }
}

fn doomed(sim: &Sim, new_road: u32, split_nodes: &[u32]) -> u64 {
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .filter(|&slot| match vehicles.place[slot as usize] {
            Place::Link { link, .. } => road_of(link) == new_road,
            Place::Movement { node, .. } => {
                split_nodes.contains(&node)
                    || [0, 1].iter().any(|&k| {
                        vehicles
                            .route_link(slot, k)
                            .is_some_and(|link| road_of(link) == new_road)
                    })
            }
        })
        .count() as u64
}

fn max_link_ok(sim: &Sim) -> bool {
    let limit = 2 * sim.network().roads.count() as LinkId;
    let vehicles = sim.vehicles();
    vehicles
        .live_slots()
        .all(|slot| vehicles.route(slot).iter().all(|&link| link < limit))
}

fn run_rows(sim: &mut Sim, ticks: u32) {
    for _ in 0..ticks {
        feed_rows(sim);
        sim.step();
        check_invariants(sim);
        assert!(max_link_ok(sim));
    }
}

fn mid_row(road: u32) -> Endpoint {
    Endpoint::OnRoad { road, at_m: 75.0 }
}

type SplitShape = (Vec<Vec2>, f64, u32, u32);

fn shapes(sim: &Sim, roads: [u32; 2]) -> [SplitShape; 2] {
    roads.map(|road| split_geometry(sim.network(), road))
}

#[test]
fn add_road_undo() {
    let mut sim = city(2_000.0);
    run_rows(&mut sim, 300);
    let hash = sim.network().content_hash();
    let before = counts(&sim);
    let roads = [row_road(2, 2), row_road(2, 3)];
    let geometry = shapes(&sim, roads);
    ok(apply_now(
        &mut sim,
        add(mid_row(roads[0]), mid_row(roads[1]), None, 0),
    ));
    assert_eq!(counts(&sim), (before.0 + 3, before.1 + 2));
    run_rows(&mut sim, 100);
    let split_nodes = [before.1 as u32, before.1 as u32 + 1];
    let expected = doomed(&sim, before.0 as u32 + 2, &split_nodes);
    let stranded = sim.stranded();
    ok(apply_now(&mut sim, EditCommand::Undo));
    assert_eq!(sim.stranded() - stranded, expected);
    assert_eq!(sim.network().content_hash(), hash);
    assert_eq!(counts(&sim), before);
    assert_eq!(shapes(&sim, roads), geometry);
    assert!(max_link_ok(&sim));
    assert_routes_connected(&sim);
    run_rows(&mut sim, 200);
}

fn with(from: Endpoint, to: Endpoint) -> EditCommand {
    add(from, to, None, 0)
}

fn lanes(forward: u8, backward: u8) -> EditCommand {
    EditCommand::AddRoad {
        from: at_node(2, 2),
        to: at_node(3, 3),
        via: None,
        lanes_forward: forward,
        lanes_backward: backward,
        layer: 0,
    }
}

fn diagonal(via: Option<[f64; 2]>, layer: i8) -> EditCommand {
    add(at_node(2, 2), at_node(3, 3), via, layer)
}

fn shape_cases() -> Vec<(EditCommand, EditError)> {
    vec![
        (
            with(Endpoint::Node { node: 9_999 }, at_node(3, 3)),
            EditError::NodeNotFound,
        ),
        (
            with(
                Endpoint::OnRoad {
                    road: 9_999,
                    at_m: 50.0,
                },
                at_node(3, 3),
            ),
            EditError::RoadNotFound,
        ),
        (lanes(0, 0), EditError::InvalidLanes),
        (lanes(5, 0), EditError::InvalidLanes),
        (diagonal(None, 3), EditError::InvalidLayer),
        (diagonal(None, -1), EditError::InvalidLayer),
        (
            with(at_node(2, 2), on_split_road(5.0)),
            EditError::TooCloseToEnd,
        ),
        (
            with(at_node(2, 2), on_split_road(f64::NAN)),
            EditError::TooCloseToEnd,
        ),
    ]
}

fn geometry_cases() -> Vec<(EditCommand, EditError)> {
    vec![
        (with(at_node(2, 2), at_node(2, 2)), EditError::SameEndpoint),
        (
            with(on_split_road(40.0), on_split_road(90.0)),
            EditError::SameEndpoint,
        ),
        (
            with(at_node(0, 0), at_node(10, 10)),
            EditError::InvalidLength,
        ),
        (diagonal(Some([5_000.0, 0.0]), 0), EditError::InvalidLength),
        (diagonal(Some([f64::NAN, 0.0]), 0), EditError::InvalidLength),
        (with(at_node(2, 2), at_node(2, 4)), EditError::AngleTooSharp),
        (with(at_node(2, 2), at_node(4, 3)), EditError::CrossesRoad),
    ]
}

#[test]
fn add_road_errors() {
    let mut sim = city(0.0);
    for (command, error) in shape_cases().into_iter().chain(geometry_cases()) {
        assert_eq!(err(&mut sim, command.clone()), error, "{command:?}");
    }
}

#[test]
fn add_road_errors_isolated_deleted_region() {
    let mut sim = city(0.0);
    ok(apply_now(&mut sim, EditCommand::DeleteRoad { road: 0 }));
    let corner_row = EditCommand::DeleteRoad {
        road: row_road(0, 0),
    };
    ok(apply_now(&mut sim, corner_row));
    let isolated = with(at_node(0, 0), at_node(1, 1));
    assert_eq!(err(&mut sim, isolated), EditError::EndpointIsolated);
    let deleted = with(
        Endpoint::OnRoad {
            road: 0,
            at_m: 50.0,
        },
        at_node(1, 1),
    );
    assert_eq!(err(&mut sim, deleted), EditError::RoadDeleted);
    sim.network_mut().set_region(Some(Region {
        center: Vec2::new(375.0, 375.0),
        radius: 200.0,
    }));
    let outside = with(at_node(2, 2), at_node(6, 6));
    assert_eq!(err(&mut sim, outside), EditError::OutsideRegion);
    ok(apply_now(&mut sim, diagonal(None, 0)));
}

#[test]
fn curved_geometry() {
    let mut sim = city(0.0);
    let via = [395.0, 355.0];
    let new_road = sim.network().roads.count() as u32;
    ok(apply_now(
        &mut sim,
        add(at_node(2, 2), at_node(3, 3), Some(via), 1),
    ));
    let network = sim.network();
    let points = network.roads.points(new_road);
    assert_eq!(points.len(), 17);
    assert_eq!(points[0], network.nodes.pos[node(2, 2) as usize]);
    assert_eq!(points[16], network.nodes.pos[node(3, 3) as usize]);
    let curve = QuadraticBezier {
        p0: points[0],
        p1: Vec2::new(via[0], via[1]),
        p2: points[16],
    };
    assert_eq!(points[8], curve.point(0.5));
}

const CURVE_FIXTURE: &str = include_str!("../../../web/src/edit/tools/curve-fixture.json");

fn fixture_point(value: &serde_json::Value) -> Vec2 {
    let x = value[0].as_f64().must("x");
    let y = value[1].as_f64().must("y");
    Vec2::new(x, y)
}

#[test]
fn curve_matches_web_fixture() {
    let fixture: serde_json::Value = serde_json::from_str(CURVE_FIXTURE).must("fixture");
    let points = curve_points(
        fixture_point(&fixture["from"]),
        fixture_point(&fixture["to"]),
        Some(fixture_point(&fixture["via"])),
    );
    let expected: Vec<Vec2> = fixture["points"]
        .as_array()
        .must("points")
        .iter()
        .map(fixture_point)
        .collect();
    assert_eq!(points.len(), expected.len());
    for (got, want) in points.iter().zip(&expected) {
        assert!(got.distance(*want) < 1e-9, "{got:?} vs {want:?}");
    }
}
