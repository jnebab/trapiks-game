use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{CommandResult, EditCommand, Outcome};
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::map::{Control, MapData};
use trapiks_sim_core::network::{Network, road_of};
use trapiks_sim_core::sim::Sim;

const TURN_NODE: u32 = 220;
const FLYOVER_NODE: u32 = 290;
const ROUNDABOUT_NODE: u32 = 250;

fn map() -> MapData {
    grid_city(&GridCity {
        cols: 20,
        rows: 20,
        spacing: 150.0,
    })
}

fn straight_pair(network: &Network, node: u32) -> [u32; 2] {
    let roads = &network.nodes.roads[node as usize];
    let direction = |road: u32| {
        let link = network.departing_link(road, node);
        network.centre_pose(link, 10.0).1
    };
    let first = roads[0];
    let opposite = roads[1..]
        .iter()
        .copied()
        .find(|&road| direction(first).dot(direction(road)) < -0.9)
        .unwrap_or(first);
    [first, opposite]
}

fn edits(map: &MapData) -> Vec<EditCommand> {
    let network = Network::from_map(map);
    let (from, to, _) = network.turns(TURN_NODE)[0];
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
        EditCommand::BuildFlyover {
            node: FLYOVER_NODE,
            through: straight_pair(&network, FLYOVER_NODE),
        },
        EditCommand::Undo,
        EditCommand::BuildRoundabout {
            node: ROUNDABOUT_NODE,
            radius_m: 18,
        },
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
    for tick in 0..900u64 {
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
    assert_eq!(results_a.len(), 8);
    for result in &results_a {
        assert!(matches!(result.outcome, Outcome::Ok(_)), "{result:?}");
    }
}
