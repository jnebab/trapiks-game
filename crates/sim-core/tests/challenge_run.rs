mod support;

use support::Must;
use trapiks_sim_core::challenge::{Challenge, ChallengeRun, RunResult, RunState};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{EditCommand, Outcome};
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::map::{Control, MapData};
use trapiks_sim_core::sim::Sim;

const ROWS: u32 = 30;
const PLAIN_NODE: u32 = 11 * (ROWS + 1) + 11;
const SIGNAL_NODE: u32 = 10 * (ROWS + 1) + 10;
const AVENUE_10: [u32; 2] = [10 * ROWS + 9, 10 * ROWS + 10];
const DELETED_ROAD: u32 = 12 * ROWS + 12;

fn city() -> MapData {
    grid_city(&GridCity {
        cols: 30,
        rows: ROWS,
        spacing: 150.0,
    })
}

fn downtown() -> Challenge {
    let text = include_str!("../../../web/public/challenges/synthetic.json");
    let all: Vec<Challenge> = serde_json::from_str(text).must("json");
    all.into_iter()
        .find(|c| c.id == "downtown")
        .must("downtown")
}

fn roomy_downtown() -> Challenge {
    Challenge {
        budget: 20_000,
        ..downtown()
    }
}

fn finish(run: &mut ChallengeRun) -> RunResult {
    match run.advance(u32::MAX) {
        RunState::Finished(result) => result,
        RunState::Running { .. } => panic!("run did not finish"),
    }
}

fn short_run(map: &MapData, challenge: &Challenge, commands: &[EditCommand]) -> RunResult {
    let mut run = ChallengeRun::with_ticks(map, challenge, commands, 300, 600).must("run");
    finish(&mut run)
}

#[test]
fn baseline_deterministic() {
    let map = city();
    let first = short_run(&map, &downtown(), &[]);
    let second = short_run(&map, &downtown(), &[]);
    assert_eq!(first, second);
    assert!(first.created > 0);
    println!("downtown short baseline: {first:?}");
}

#[test]
fn incremental_equals_bulk() {
    let map = city();
    let mut stepped = ChallengeRun::with_ticks(&map, &downtown(), &[], 300, 600).must("run");
    let mut last = stepped.advance(1);
    while let RunState::Running {
        done_ticks,
        total_ticks,
    } = last
    {
        assert!(done_ticks < total_ticks);
        last = stepped.advance(1);
    }
    assert_eq!(stepped.advance(5), last);
    assert_eq!(last, RunState::Finished(short_run(&map, &downtown(), &[])));
}

fn live_sim(map: &MapData, challenge: &Challenge) -> Sim {
    let (center_x, center_y) = challenge.center_xy(map);
    let config = SimConfig {
        seed: challenge.seed,
        mode: SimMode::Region {
            center_x,
            center_y,
            radius: challenge.radius_m,
        },
        vehicles_per_hour: challenge.vehicles_per_hour,
        budget: Some(challenge.budget),
    };
    Sim::from_config(map, &config)
}

fn run_to(sim: &mut Sim, tick: u64) {
    while sim.tick() < tick {
        sim.step();
    }
}

fn apply_at(sim: &mut Sim, tick: u64, command: EditCommand) {
    run_to(sim, tick);
    let seq = sim.enqueue(command);
    sim.step();
    let result = sim
        .take_results()
        .into_iter()
        .find(|r| r.seq == seq)
        .must("result");
    assert!(matches!(result.outcome, Outcome::Ok(_)), "{result:?}");
}

fn timing_for(sim: &Sim) -> EditCommand {
    let signal = sim
        .inspect_node(PLAIN_NODE)
        .and_then(|n| n.signal)
        .must("signal");
    EditCommand::SetSignalTiming {
        node: PLAIN_NODE,
        greens_s: vec![20; signal.phase_count as usize],
        offset_s: 5,
    }
}

fn edited_live(map: &MapData, challenge: &Challenge) -> Sim {
    let mut sim = live_sim(map, challenge);
    let control = EditCommand::SetJunctionControl {
        node: PLAIN_NODE,
        control: Control::Signal,
    };
    apply_at(&mut sim, 50, control);
    let timing = timing_for(&sim);
    apply_at(&mut sim, 400, timing);
    let flyover = EditCommand::BuildFlyover {
        node: SIGNAL_NODE,
        through: AVENUE_10,
    };
    apply_at(&mut sim, 900, flyover);
    apply_at(
        &mut sim,
        1300,
        EditCommand::DeleteRoad { road: DELETED_ROAD },
    );
    apply_at(&mut sim, 2000, EditCommand::Undo);
    sim
}

#[test]
fn replay_matches_live() {
    let map = city();
    let challenge = roomy_downtown();
    assert_eq!(map.nodes.control[PLAIN_NODE as usize], Control::Priority);
    let live = edited_live(&map, &challenge);
    let log = live.command_log().to_vec();
    assert_eq!(log.len(), 5);
    let run = ChallengeRun::with_ticks(&map, &challenge, &log, 300, 600).must("replay");
    let (a, b) = (live.network(), run.sim().network());
    assert_eq!(run.sim().tick(), 0);
    assert_eq!(a.content_hash(), b.content_hash());
    assert_eq!(a.roads.count(), b.roads.count());
    assert_eq!(a.nodes.count(), b.nodes.count());
    for road in 0..a.roads.count() as u32 {
        assert_eq!(a.roads.points(road), b.roads.points(road), "road {road}");
    }
    let first = short_run(&map, &challenge, &log);
    let second = short_run(&map, &challenge, &log);
    assert_eq!(first, second);
}

#[test]
fn replay_rejects_failing_command() {
    let map = city();
    let result = ChallengeRun::with_ticks(&map, &downtown(), &[EditCommand::Undo], 300, 600);
    assert!(result.is_err());
}

#[test]
fn log_compacts_demand() {
    let map = city();
    let mut sim = Sim::new(&map, 1);
    for vph in [100.0, 200.0] {
        sim.enqueue(EditCommand::SetDemand {
            vehicles_per_hour: vph,
        });
    }
    sim.enqueue(EditCommand::DeleteRoad { road: DELETED_ROAD });
    sim.enqueue(EditCommand::SetDemand {
        vehicles_per_hour: 300.0,
    });
    sim.enqueue(EditCommand::Undo);
    sim.enqueue(EditCommand::Undo);
    sim.apply_queued();
    assert_eq!(
        sim.command_log(),
        &[
            EditCommand::SetDemand {
                vehicles_per_hour: 300.0
            },
            EditCommand::DeleteRoad { road: DELETED_ROAD },
            EditCommand::Undo,
        ]
    );
}
