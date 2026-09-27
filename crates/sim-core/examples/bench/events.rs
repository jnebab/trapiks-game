use trapiks_sim_core::edit::{EditCommand, Endpoint, QuoteOutcome};
use trapiks_sim_core::map::RoadClass;
use trapiks_sim_core::network::{Direction, link_id, road_of};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const ROUNDABOUT_RADIUS_M: u8 = 18;
const MIN_CONNECT_M: f64 = 30.0;
const CONNECT_CANDIDATES: usize = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    DeleteRoad,
    Roundabout,
    AddRoad,
}

#[derive(Clone, Copy, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub tick: u64,
}

pub struct Planned {
    pub label: String,
    pub vehicles: usize,
}

pub fn plan(sim: &Sim, kind: EventKind) -> Option<(EditCommand, Planned)> {
    match kind {
        EventKind::DeleteRoad => delete_busiest_primary(sim),
        EventKind::Roundabout => roundabout_on_busiest_node(sim),
        EventKind::AddRoad => connect_most_used_road(sim),
    }
}

fn road_counts(sim: &Sim) -> Vec<usize> {
    let vehicles = sim.vehicles();
    let mut counts = vec![0usize; sim.network().roads.count()];
    for slot in vehicles.live_slots() {
        if let Place::Link { link, .. } = vehicles.place[slot as usize] {
            counts[road_of(link) as usize] += 1;
        }
    }
    counts
}

fn busiest(counts: &[usize], eligible: impl Fn(usize) -> bool) -> Option<(u32, usize)> {
    counts
        .iter()
        .enumerate()
        .filter(|&(index, _)| eligible(index))
        .max_by_key(|&(index, &count)| (count, std::cmp::Reverse(index)))
        .map(|(index, &count)| (index as u32, count))
}

fn delete_busiest_primary(sim: &Sim) -> Option<(EditCommand, Planned)> {
    let network = sim.network();
    let eligible = |road: usize| {
        network.roads.class[road] == RoadClass::Primary && network.roads.is_live(road as u32)
    };
    let (road, vehicles) = busiest(&road_counts(sim), eligible)?;
    let planned = Planned {
        label: format!("delete road {road}"),
        vehicles,
    };
    Some((EditCommand::DeleteRoad { road }, planned))
}

fn roundabout_on_busiest_node(sim: &Sim) -> Option<(EditCommand, Planned)> {
    let network = sim.network();
    let roads = road_counts(sim);
    let nodes: Vec<usize> = (0..network.nodes.count())
        .map(|node| {
            network.nodes.roads[node]
                .iter()
                .map(|&r| roads[r as usize])
                .sum()
        })
        .collect();
    let (node, vehicles) = busiest(&nodes, |node| network.active_degree(node as u32) >= 3)?;
    let command = EditCommand::BuildRoundabout {
        node,
        radius_m: ROUNDABOUT_RADIUS_M,
    };
    let planned = Planned {
        label: format!("roundabout at node {node}"),
        vehicles,
    };
    Some((command, planned))
}

fn connect_most_used_road(sim: &Sim) -> Option<(EditCommand, Planned)> {
    let network = sim.network();
    let (road, vehicles) = busiest(&road_counts(sim), |road| {
        network.roads.is_live(road as u32) && network.roads.length[road] >= 2.0 * MIN_CONNECT_M
    })?;
    let at_m = network.roads.length[road as usize] / 2.0;
    let (centre, _) = network.centre_pose(link_id(road, Direction::Forward), at_m);
    let mut candidates: Vec<(f64, u32)> = (0..network.nodes.count() as u32)
        .filter(|&node| network.active_degree(node) > 0)
        .map(|node| (network.nodes.pos[node as usize].distance(centre), node))
        .filter(|&(distance, _)| distance >= MIN_CONNECT_M)
        .collect();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let command = candidates
        .iter()
        .take(CONNECT_CANDIDATES)
        .map(|&(_, node)| add_road(road, at_m, node))
        .find(|command| matches!(sim.quote(command), QuoteOutcome::Ok(_)))?;
    let planned = Planned {
        label: format!("split road {road} and connect"),
        vehicles,
    };
    Some((command, planned))
}

fn add_road(road: u32, at_m: f64, node: u32) -> EditCommand {
    EditCommand::AddRoad {
        from: Endpoint::OnRoad { road, at_m },
        to: Endpoint::Node { node },
        via: None,
        lanes_forward: 1,
        lanes_backward: 1,
        layer: 0,
    }
}
