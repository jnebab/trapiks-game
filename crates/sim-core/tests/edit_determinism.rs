use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{CommandResult, EditCommand, Outcome};
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::map::{Control, MapData};
use trapiks_sim_core::network::{Network, road_of};
use trapiks_sim_core::sim::Sim;

const TURN_NODE: u32 = 220;

fn map() -> MapData {
    grid_city(&GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    })
}

fn edits(map: &MapData) -> Vec<EditCommand> {
    let (from, to, _) = Network::from_map(map).turns(TURN_NODE)[0];
    vec![
        EditCommand::DeleteRoad { road: 150 },
        EditCommand::SetLanes {
            road: 623,
            forward: 3,
            backward: 1,
        },
        EditCommand::SetSpeedLimit { road: 528, kph: 30 },
        EditCommand::SetJunctionControl {
            node: 176,
            control: Control::AllWayStop,
        },
        EditCommand::SetTurnAllowed {
            node: TURN_NODE,
            from_road: road_of(from),
            to_road: road_of(to),
            allowed: false,
        },
        EditCommand::Undo,
    ]
}

fn run(map: &MapData) -> (u64, Vec<CommandResult>) {
    let config = SimConfig {
        seed: 5,
        mode: SimMode::City,
        vehicles_per_hour: 8_000.0,
        budget: None,
    };
    let mut sim = Sim::from_config(map, &config);
    let mut pending = edits(map).into_iter();
    let mut results = Vec::new();
    for tick in 0..700u64 {
        if tick > 0
            && tick.is_multiple_of(100)
            && let Some(command) = pending.next()
        {
            sim.enqueue(command);
        }
        sim.step();
        results.extend(sim.take_results());
    }
    (sim.state_hash(), results)
}

#[test]
fn determinism_with_edits() {
    let map = map();
    let (hash_a, results_a) = run(&map);
    let (hash_b, results_b) = run(&map);
    assert_eq!(hash_a, hash_b);
    assert_eq!(results_a, results_b);
    assert_eq!(results_a.len(), 6);
    for result in &results_a {
        assert!(matches!(result.outcome, Outcome::Ok(_)), "{result:?}");
    }
}
