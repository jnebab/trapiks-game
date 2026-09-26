mod support;

use support::{apply_now, grid_10};
use trapiks_sim_core::edit::{EditCommand, EditOutcome, Outcome};
use trapiks_sim_core::map::Control;
use trapiks_sim_core::network::{Network, road_of};
use trapiks_sim_core::sim::Sim;

const RESIDENTIAL_ROAD: u32 = 12;
const PLAIN_NODE: u32 = 13;
const SIGNAL_NODE: u32 = 60;

fn turn_command(network: &Network) -> EditCommand {
    let (from, to, _) = network.turns(PLAIN_NODE)[0];
    EditCommand::SetTurnAllowed {
        node: PLAIN_NODE,
        from_road: road_of(from),
        to_road: road_of(to),
        allowed: false,
    }
}

fn commands(network: &Network) -> Vec<EditCommand> {
    let phases = network
        .signals()
        .cluster_at(SIGNAL_NODE)
        .map_or(0, |cluster| cluster.phases.len());
    vec![
        EditCommand::DeleteRoad {
            road: RESIDENTIAL_ROAD,
        },
        EditCommand::SetLanes {
            road: RESIDENTIAL_ROAD,
            forward: 2,
            backward: 0,
        },
        EditCommand::SetSpeedLimit {
            road: RESIDENTIAL_ROAD,
            kph: 40,
        },
        EditCommand::SetJunctionControl {
            node: PLAIN_NODE,
            control: Control::Signal,
        },
        EditCommand::SetSignalTiming {
            node: SIGNAL_NODE,
            greens_s: vec![20; phases],
            offset_s: 7,
        },
        turn_command(network),
    ]
}

fn expect_ok(outcome: Outcome) -> EditOutcome {
    match outcome {
        Outcome::Ok(outcome) => outcome,
        Outcome::Err(error) => panic!("unexpected {error:?}"),
    }
}

fn roundtrip(command: EditCommand) {
    let map = grid_10();
    let mut sim = Sim::new(&map, 3);
    let original = sim.network().content_hash();
    let spent = sim.budget().spent;
    let applied = expect_ok(apply_now(&mut sim, command.clone()));
    assert_ne!(sim.network().content_hash(), original, "{command:?}");
    assert!(applied.cost > 0);
    let undone = expect_ok(apply_now(&mut sim, EditCommand::Undo));
    assert_eq!(sim.network().content_hash(), original, "{command:?}");
    assert_eq!(sim.budget().spent, spent);
    assert_eq!(undone.cost, -applied.cost);
    assert_eq!(undone.changed_roads, applied.changed_roads);
    assert_eq!(undone.changed_nodes, applied.changed_nodes);
    assert_eq!(sim.undo_depth(), 0);
}

#[test]
fn edit_roundtrip() {
    let network = Network::from_map(&grid_10());
    for command in commands(&network) {
        roundtrip(command);
    }
}

#[test]
fn road_edit_reports_road_and_endpoints() {
    let map = grid_10();
    let mut sim = Sim::new(&map, 3);
    let road = RESIDENTIAL_ROAD as usize;
    let mut ends = vec![map.roads.from[road], map.roads.to[road]];
    ends.sort_unstable();
    let outcome = expect_ok(apply_now(
        &mut sim,
        EditCommand::SetSpeedLimit {
            road: RESIDENTIAL_ROAD,
            kph: 50,
        },
    ));
    assert_eq!(outcome.changed_roads, vec![RESIDENTIAL_ROAD]);
    assert_eq!(outcome.changed_nodes, ends);
    assert_eq!(outcome.cost, 10);
}
