use trapiks_sim_core::edit::EditCommand;
use trapiks_sim_core::map::Control;
use trapiks_sim_core::network::{Direction, Junction, TurnKind, direction_of, road_of};
use trapiks_sim_core::sim::Sim;

use crate::baseline::Samples;
use crate::probe::Probe;

const LONG_GREEN_S: u32 = 50;
const SHORT_GREEN_S: u32 = 20;
const EVEN_GREEN_S: u32 = 30;
const ROUNDABOUT_RADII_M: [u8; 2] = [20, 30];
const MAX_LANES: u8 = 6;

pub fn flyovers(sim: &Sim, node: u32) -> Vec<Probe> {
    let pairs = sim
        .inspect_node(node)
        .map_or_else(Vec::new, |n| n.flyover_pairs);
    pairs
        .into_iter()
        .map(|through| Probe {
            label: format!("flyover through roads {through:?}"),
            commands: vec![EditCommand::BuildFlyover { node, through }],
        })
        .collect()
}

pub fn signal_prefix(sim: &Sim, node: u32) -> Vec<EditCommand> {
    let signalized = sim
        .inspect_node(node)
        .is_some_and(|n| n.control == Control::Signal);
    if signalized {
        return Vec::new();
    }
    vec![EditCommand::SetJunctionControl {
        node,
        control: Control::Signal,
    }]
}

pub fn green_splits(phases: usize) -> Vec<Vec<u32>> {
    let mut splits = vec![vec![EVEN_GREEN_S; phases]];
    for favoured in 0..phases {
        let split = (0..phases)
            .map(|phase| {
                if phase == favoured {
                    LONG_GREEN_S
                } else {
                    SHORT_GREEN_S
                }
            })
            .collect();
        splits.push(split);
    }
    splits
}

pub fn left_bans(junction: &Junction, node: u32, samples: &Samples) -> Vec<Probe> {
    let mut lefts: Vec<(u64, u16)> = samples
        .movements
        .iter()
        .filter(|&(&m, _)| is_left(junction, m))
        .map(|(&m, &count)| (count, m))
        .collect();
    lefts.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let bans: Vec<EditCommand> = lefts
        .iter()
        .take(2)
        .map(|&(_, m)| ban(junction, node, m))
        .collect();
    let mut probes: Vec<Probe> = bans
        .first()
        .map(|first| Probe {
            label: format!("ban busiest left ({} samples)", lefts[0].0),
            commands: vec![first.clone()],
        })
        .into_iter()
        .collect();
    if bans.len() == 2 {
        probes.push(Probe {
            label: "ban two busiest lefts".to_string(),
            commands: bans,
        });
    }
    probes
}

fn is_left(junction: &Junction, movement: u16) -> bool {
    junction
        .movements
        .get(usize::from(movement))
        .is_some_and(|m| m.kind == TurnKind::Left)
}

fn ban(junction: &Junction, node: u32, movement: u16) -> EditCommand {
    let m = &junction.movements[usize::from(movement)];
    EditCommand::SetTurnAllowed {
        node,
        from_road: road_of(m.from_link),
        to_road: road_of(m.to_link),
        allowed: false,
    }
}

pub fn extra_lane(sim: &Sim, samples: &Samples) -> Option<Probe> {
    let (&link, &count) = samples
        .approaches
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))?;
    let road = road_of(link);
    let info = sim.inspect_road(road)?;
    let (mut forward, mut backward) = (info.lanes_forward, info.lanes_backward);
    match direction_of(link) {
        Direction::Forward => forward = (forward + 1).min(MAX_LANES),
        Direction::Backward => backward = (backward + 1).min(MAX_LANES),
    }
    Some(Probe {
        label: format!("add lane on approach road {road} ({count} samples)"),
        commands: vec![EditCommand::SetLanes {
            road,
            forward,
            backward,
        }],
    })
}

pub fn roundabouts(sim: &Sim, node: u32) -> Vec<Probe> {
    let plain = sim
        .inspect_node(node)
        .is_some_and(|n| n.control != Control::Signal);
    if !plain {
        return Vec::new();
    }
    ROUNDABOUT_RADII_M
        .iter()
        .map(|&radius_m| Probe {
            label: format!("roundabout radius {radius_m} m"),
            commands: vec![EditCommand::BuildRoundabout { node, radius_m }],
        })
        .collect()
}
