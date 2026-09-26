mod support;

use support::apply_now;
use trapiks_sim_core::edit::{EditCommand, EditOutcome, Outcome};
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::network::Network;
use trapiks_sim_core::render::{JunctionShapes, NetworkDelta, junction_shapes, network_delta};
use trapiks_sim_core::sim::Sim;

fn ok(outcome: Outcome) -> EditOutcome {
    match outcome {
        Outcome::Ok(outcome) => outcome,
        Outcome::Err(error) => panic!("edit failed: {error:?}"),
    }
}

fn ring_of(shapes: &JunctionShapes, index: usize) -> Vec<(f32, f32)> {
    let start = shapes.ring_start[index] as usize;
    let end = shapes.ring_start[index + 1] as usize;
    (start..end).map(|i| (shapes.x[i], shapes.y[i])).collect()
}

fn north_south_reach(ring: &[(f32, f32)]) -> f32 {
    ring.iter().map(|&(_, y)| y.abs()).fold(0.0, f32::max)
}

fn delta_for(sim: &Sim, outcome: &EditOutcome) -> NetworkDelta {
    network_delta(
        sim.network(),
        &outcome.changed_roads,
        &outcome.changed_nodes,
    )
}

#[test]
fn delta_contents() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let before = junction_shapes(sim.network());
    let command = EditCommand::SetLanes {
        road: 1,
        forward: 3,
        backward: 3,
    };
    let outcome = ok(apply_now(&mut sim, command));
    let delta = delta_for(&sim, &outcome);
    assert_rows(&delta);
    assert_shapes(&delta, &before);
}

fn assert_rows(delta: &NetworkDelta) {
    assert_eq!(delta.roads.id, vec![1]);
    assert_eq!(delta.roads.lanes_forward, vec![3]);
    assert_eq!(delta.roads.lanes_backward, vec![3]);
    assert_eq!(delta.nodes.id, vec![0, 2]);
    assert_eq!(delta.setbacks.id, vec![0, 1, 2, 3]);
}

fn assert_shapes(delta: &NetworkDelta, before: &JunctionShapes) {
    assert_eq!(delta.junctions.node, vec![0, 2]);
    let centre = ring_of(&delta.junctions, 0);
    assert!(north_south_reach(&centre) > north_south_reach(&ring_of(before, 0)));
    assert!(ring_of(&delta.junctions, 1).is_empty());
    assert_eq!(delta.markers.nodes, vec![0, 2]);
    assert_eq!(delta.markers.markers.node, vec![0; 4]);
}

fn counts(network: &Network) -> (u32, u32) {
    (network.roads.count() as u32, network.nodes.count() as u32)
}

#[test]
fn delta_after_flyover_undo() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    let original = counts(sim.network());
    let flyover = EditCommand::BuildFlyover {
        node: 0,
        through: [2, 0],
    };
    let built = ok(apply_now(&mut sim, flyover));
    let undone = ok(apply_now(&mut sim, EditCommand::Undo));
    let mut roads = built.changed_roads.clone();
    roads.extend(&undone.changed_roads);
    let mut nodes = built.changed_nodes.clone();
    nodes.extend(&undone.changed_nodes);
    let delta = network_delta(sim.network(), &roads, &nodes);
    assert_eq!((delta.road_count, delta.node_count), original);
    assert!(delta.roads.id.iter().all(|&id| id < original.0));
    assert!(delta.setbacks.id.iter().all(|&id| id < original.0));
    assert!(delta.nodes.id.iter().all(|&id| id < original.1));
    assert!(delta.junctions.node.iter().all(|&id| id < original.1));
    assert!(!delta.roads.id.is_empty());
}
