mod support;

use support::Must;
use trapiks_sim_core::challenge::{Center, Challenge, ChallengeRun, RunResult, RunState, score};
use trapiks_sim_core::demand::DemandTables;
use trapiks_sim_core::edit::EditCommand;
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::{MapData, RoadClass};
use trapiks_sim_core::network::{Network, Region, road_of};
use trapiks_sim_core::routing::{LinkCosts, RouteGraph};

fn city(size: u32) -> MapData {
    grid_city(&GridCity {
        cols: size,
        rows: size,
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

fn small_challenge() -> Challenge {
    Challenge {
        id: "small".into(),
        name: "Small".into(),
        blurb: String::new(),
        center: Center::Local { x: 750.0, y: 750.0 },
        radius_m: 400.0,
        vehicles_per_hour: 3000.0,
        budget: 20_000,
        ..downtown()
    }
}

fn evaluate(map: &MapData, challenge: &Challenge, commands: &[EditCommand]) -> RunResult {
    let mut run = ChallengeRun::with_ticks(map, challenge, commands, 600, 1200).must("run");
    match run.advance(u32::MAX) {
        RunState::Finished(result) => result,
        RunState::Running { .. } => panic!("run did not finish"),
    }
}

#[test]
fn flyover_improves() {
    let map = city(10);
    let challenge = small_challenge();
    let flyover = EditCommand::BuildFlyover {
        node: 5 * 11 + 5,
        through: [5 * 10 + 4, 5 * 10 + 5],
    };
    let baseline = evaluate(&map, &challenge, &[]);
    let after = evaluate(&map, &challenge, &[flyover]);
    println!("flyover baseline: {baseline:?}");
    println!("flyover after: {after:?}");
    assert!(after.mean_delay_s < baseline.mean_delay_s);
}

#[test]
fn speed_limit_cut_does_not_improve() {
    let map = city(30);
    let challenge = downtown();
    let network = regional(&map, &challenge);
    let cuts: Vec<EditCommand> = (0..network.roads.count() as u32)
        .filter(|&road| network.is_road_active(road))
        .filter(|&road| network.roads.class[road as usize] == RoadClass::Primary)
        .map(|road| EditCommand::SetSpeedLimit { road, kph: 30 })
        .collect();
    let baseline = evaluate(&map, &challenge, &[]);
    println!("speed cuts: {}", cuts.len());
    let after = evaluate(&map, &challenge, &cuts);
    let result = score(&challenge, &baseline, &after, 0);
    println!(
        "speed cut: {baseline:?} -> {after:?}, improvement {}",
        result.improvement
    );
    assert!(result.improvement <= 0.02);
}

fn regional(map: &MapData, challenge: &Challenge) -> Network {
    let mut network = Network::from_map(map);
    let (x, y) = challenge.center_xy(map);
    network.set_region(Some(Region {
        center: Vec2::new(x, y),
        radius: challenge.radius_m,
    }));
    network
}

fn top_source_roads(map: &MapData, challenge: &Challenge, count: usize) -> Vec<u32> {
    let network = regional(map, challenge);
    let graph = RouteGraph::build(&network);
    let costs = LinkCosts::build(&network);
    let tables = DemandTables::build(&network, &graph, &costs);
    let sources = &network.region().must("region").sources;
    let origins = &tables.origins;
    let mut weighted: Vec<(f64, u32)> = origins
        .links
        .iter()
        .enumerate()
        .filter(|(_, link)| sources.contains(link))
        .map(|(i, &link)| {
            let before = if i == 0 {
                0.0
            } else {
                origins.cumulative[i - 1]
            };
            (origins.cumulative[i] - before, road_of(link))
        })
        .collect();
    weighted.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut roads: Vec<u32> = Vec::new();
    for (_, road) in weighted {
        if !roads.contains(&road) && roads.len() < count {
            roads.push(road);
        }
    }
    roads
}

#[test]
fn deleting_access_roads_is_penalized() {
    let map = city(30);
    let challenge = downtown();
    let deletes: Vec<EditCommand> = top_source_roads(&map, &challenge, 3)
        .into_iter()
        .map(|road| EditCommand::DeleteRoad { road })
        .collect();
    assert_eq!(deletes.len(), 3);
    let baseline = evaluate(&map, &challenge, &[]);
    let after = evaluate(&map, &challenge, &deletes);
    let result = score(&challenge, &baseline, &after, 0);
    println!(
        "access deletes {deletes:?}: {baseline:?} -> {after:?}, improvement {}",
        result.improvement
    );
    assert!(result.improvement < 0.0);
}
