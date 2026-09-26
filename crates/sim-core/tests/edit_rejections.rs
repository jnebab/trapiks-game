mod support;

use support::{apply_now, grid_10};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{EditCommand, EditError, Outcome};
use trapiks_sim_core::map::Control;
use trapiks_sim_core::sim::Sim;

const CORNER_NODE: u32 = 0;
const PLAIN_NODE: u32 = 13;
const SIGNAL_NODE: u32 = 60;
const ROAD: u32 = 12;

fn city() -> Sim {
    Sim::new(&grid_10(), 1)
}

fn region() -> Sim {
    let config = SimConfig {
        seed: 1,
        mode: SimMode::Region {
            center_x: 750.0,
            center_y: 750.0,
            radius: 300.0,
        },
        vehicles_per_hour: 0.0,
        budget: None,
    };
    Sim::from_config(&grid_10(), &config)
}

fn rejects(sim: &mut Sim, command: EditCommand, expected: EditError) {
    assert_eq!(sim.quote(&command), Err(expected), "{command:?}");
    assert_eq!(
        apply_now(sim, command.clone()),
        Outcome::Err(expected),
        "{command:?}"
    );
}

fn road_errors() {
    let mut sim = city();
    rejects(
        &mut sim,
        EditCommand::DeleteRoad { road: 99_999 },
        EditError::RoadNotFound,
    );
    let _ = apply_now(&mut sim, EditCommand::DeleteRoad { road: ROAD });
    rejects(
        &mut sim,
        EditCommand::SetSpeedLimit {
            road: ROAD,
            kph: 50,
        },
        EditError::RoadDeleted,
    );
    rejects(
        &mut sim,
        EditCommand::SetLanes {
            road: ROAD + 1,
            forward: 0,
            backward: 0,
        },
        EditError::InvalidLanes,
    );
    rejects(
        &mut sim,
        EditCommand::SetSpeedLimit {
            road: ROAD + 1,
            kph: 5,
        },
        EditError::InvalidSpeed,
    );
    rejects(
        &mut sim,
        EditCommand::SetSpeedLimit {
            road: ROAD + 1,
            kph: 20,
        },
        EditError::NoChange,
    );
}

fn node_errors() {
    let mut sim = city();
    rejects(
        &mut sim,
        EditCommand::SetJunctionControl {
            node: 99_999,
            control: Control::Stop,
        },
        EditError::NodeNotFound,
    );
    rejects(
        &mut sim,
        EditCommand::SetJunctionControl {
            node: CORNER_NODE,
            control: Control::Stop,
        },
        EditError::NotAJunction,
    );
}

fn signal_and_turn_errors() {
    let mut sim = city();
    rejects(
        &mut sim,
        EditCommand::SetSignalTiming {
            node: PLAIN_NODE,
            greens_s: vec![20, 20],
            offset_s: 0,
        },
        EditError::NotSignalized,
    );
    rejects(
        &mut sim,
        EditCommand::SetSignalTiming {
            node: SIGNAL_NODE,
            greens_s: vec![20, 20, 20, 20, 20, 20, 20],
            offset_s: 0,
        },
        EditError::InvalidTiming,
    );
    rejects(
        &mut sim,
        EditCommand::SetTurnAllowed {
            node: PLAIN_NODE,
            from_road: ROAD,
            to_road: ROAD,
            allowed: false,
        },
        EditError::TurnNotFound,
    );
}

fn session_errors() {
    let mut sim = city();
    rejects(&mut sim, EditCommand::Undo, EditError::NothingToUndo);
    rejects(
        &mut sim,
        EditCommand::SetDemand {
            vehicles_per_hour: f64::NAN,
        },
        EditError::InvalidDemand,
    );
    sim.set_budget_limit(Some(50));
    rejects(
        &mut sim,
        EditCommand::DeleteRoad { road: ROAD },
        EditError::InsufficientBudget,
    );
}

fn region_errors() {
    let mut sim = region();
    rejects(
        &mut sim,
        EditCommand::DeleteRoad { road: 0 },
        EditError::OutsideRegion,
    );
    rejects(
        &mut sim,
        EditCommand::SetJunctionControl {
            node: CORNER_NODE,
            control: Control::Stop,
        },
        EditError::OutsideRegion,
    );
    rejects(
        &mut sim,
        EditCommand::SetDemand {
            vehicles_per_hour: 100.0,
        },
        EditError::DemandLocked,
    );
}

#[test]
fn edit_rejections() {
    road_errors();
    node_errors();
    signal_and_turn_errors();
    session_errors();
    region_errors();
}
