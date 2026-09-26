use crate::fnv::Fnv64;
use crate::map::{Control, TurnBan};
use crate::network::{Network, Timing};

use super::Endpoint;

use super::add_road::{self, AddRoadSpec, AddRoadUndo};
use super::flyover::{self, FlyoverUndo};
use super::roundabout::{self, RoundaboutUndo};

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    SetDeleted {
        road: u32,
        deleted: bool,
    },
    SetLanes {
        road: u32,
        forward: u8,
        backward: u8,
    },
    SetSpeed {
        road: u32,
        kph: u8,
    },
    SetControl {
        node: u32,
        control: Control,
    },
    SetTiming {
        key: u32,
        timing: Option<Timing>,
    },
    SetBan {
        ban: TurnBan,
        banned: bool,
    },
    BuildFlyover {
        node: u32,
        through: [u32; 2],
    },
    UndoFlyover(Box<FlyoverUndo>),
    BuildRoundabout {
        node: u32,
        radius_m: u8,
    },
    UndoRoundabout(Box<RoundaboutUndo>),
    AddRoad(Box<AddRoadSpec>),
    UndoAddRoad(Box<AddRoadUndo>),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scope {
    pub roads: Vec<u32>,
    pub nodes: Vec<u32>,
    pub invalidated: Vec<u32>,
    pub signals: bool,
}

impl Edit {
    pub fn scope(&self, network: &Network) -> Scope {
        match *self {
            Edit::SetDeleted { road, .. } | Edit::SetLanes { road, .. } => {
                road_scope(network, road, true)
            }
            Edit::SetSpeed { road, .. } => road_scope(network, road, false),
            Edit::SetControl { node, .. } => node_scope(node, true),
            Edit::SetTiming { key, .. } => node_scope(key, true),
            Edit::SetBan { ban, .. } => node_scope(ban.via_node, false),
            Edit::BuildFlyover { node, through } => flyover::build_scope(network, node, through),
            Edit::UndoFlyover(ref undo) => flyover::undo_scope(network, undo),
            Edit::BuildRoundabout { node, .. } => roundabout::build_scope(network, node),
            Edit::UndoRoundabout(ref undo) => roundabout::undo_scope(network, undo),
            Edit::AddRoad(ref spec) => add_road::build_scope(network, spec),
            Edit::UndoAddRoad(ref undo) => add_road::undo_scope(network, undo),
        }
    }

    pub fn apply(&self, network: &mut Network) -> Edit {
        match *self {
            Edit::SetDeleted { road, deleted } => Edit::SetDeleted {
                road,
                deleted: network.set_road_deleted(road, deleted),
            },
            Edit::SetLanes {
                road,
                forward,
                backward,
            } => set_lanes(network, road, forward, backward),
            Edit::SetSpeed { road, kph } => Edit::SetSpeed {
                road,
                kph: network.set_road_speed(road, kph),
            },
            Edit::SetControl { node, control } => Edit::SetControl {
                node,
                control: network.set_control(node, control),
            },
            Edit::SetTiming { key, ref timing } => Edit::SetTiming {
                key,
                timing: network.set_timing(key, timing.clone()),
            },
            Edit::SetBan { ban, banned } => Edit::SetBan {
                ban,
                banned: network.set_ban(ban, banned),
            },
            _ => self.apply_structural(network),
        }
    }

    fn apply_structural(&self, network: &mut Network) -> Edit {
        match *self {
            Edit::BuildFlyover { node, through } => {
                Edit::UndoFlyover(Box::new(flyover::build(network, node, through)))
            }
            Edit::UndoFlyover(ref undo) => {
                flyover::undo(network, undo);
                Edit::BuildFlyover {
                    node: undo.node,
                    through: undo.roads,
                }
            }
            Edit::BuildRoundabout { node, radius_m } => {
                Edit::UndoRoundabout(Box::new(roundabout::build(network, node, radius_m)))
            }
            Edit::UndoRoundabout(ref undo) => {
                roundabout::undo(network, undo);
                undo.rebuild()
            }
            Edit::AddRoad(ref spec) => Edit::UndoAddRoad(Box::new(add_road::build(network, spec))),
            Edit::UndoAddRoad(ref undo) => {
                add_road::undo(network, undo);
                Edit::AddRoad(Box::new(undo.spec))
            }
            _ => self.clone(),
        }
    }

    fn hash_structural(&self, hasher: &mut Fnv64) -> bool {
        match self {
            Edit::UndoFlyover(undo) => hash_flyover_undo(undo, hasher),
            Edit::UndoRoundabout(undo) => hash_roundabout_undo(undo, hasher),
            Edit::AddRoad(spec) => {
                hasher.write_u32(10);
                hash_add_road_spec(spec, hasher);
            }
            Edit::UndoAddRoad(undo) => hash_add_road_undo(undo, hasher),
            _ => return false,
        }
        true
    }

    pub fn hash_into(&self, hasher: &mut Fnv64) {
        if self.hash_structural(hasher) {
            return;
        }
        let words = match self {
            Edit::SetDeleted { road, deleted } => vec![0, *road, u32::from(*deleted)],
            Edit::SetLanes {
                road,
                forward,
                backward,
            } => vec![1, *road, u32::from(*forward), u32::from(*backward)],
            Edit::SetSpeed { road, kph } => vec![2, *road, u32::from(*kph)],
            Edit::SetControl { node, control } => vec![3, *node, u32::from(control.code())],
            Edit::SetTiming { key, timing } => timing_words(*key, timing.as_ref()),
            Edit::SetBan { ban, banned } => {
                vec![
                    5,
                    ban.via_node,
                    ban.from_road,
                    ban.to_road,
                    u32::from(*banned),
                ]
            }
            Edit::BuildFlyover { node, through } => vec![6, *node, through[0], through[1]],
            Edit::BuildRoundabout { node, radius_m } => vec![8, *node, u32::from(*radius_m)],
            Edit::UndoFlyover(_)
            | Edit::UndoRoundabout(_)
            | Edit::AddRoad(_)
            | Edit::UndoAddRoad(_) => Vec::new(),
        };
        for word in words {
            hasher.write_u32(word);
        }
    }
}

fn hash_flyover_undo(undo: &FlyoverUndo, hasher: &mut Fnv64) {
    hasher.write_u32(7);
    hasher.write_u32(undo.node);
    for index in 0..2 {
        hasher.write_u32(undo.roads[index]);
        hasher.write_u32(undo.old_ranges[index].0);
        hasher.write_u32(undo.old_ranges[index].1);
        hasher.write_f64(undo.old_lengths[index]);
        hasher.write_u32(undo.old_ends[index]);
    }
    hasher.write_u32(undo.road_len_before);
    hasher.write_u32(undo.node_len_before);
}

fn hash_roundabout_undo(undo: &RoundaboutUndo, hasher: &mut Fnv64) {
    hasher.write_u32(9);
    hasher.write_u32(undo.node);
    hasher.write_u32(undo.arms.len() as u32);
    for arm in &undo.arms {
        hasher.write_u32(arm.road);
        hasher.write_u32(arm.old_range.0);
        hasher.write_u32(arm.old_range.1);
        hasher.write_f64(arm.old_length);
        hasher.write_u32(u32::from(arm.arrived));
        hasher.write_f64(arm.cut);
    }
    hasher.write_u32(undo.road_len_before);
    hasher.write_u32(undo.node_len_before);
}

fn hash_endpoint(endpoint: Endpoint, hasher: &mut Fnv64) {
    match endpoint {
        Endpoint::Node { node } => {
            hasher.write_u32(0);
            hasher.write_u32(node);
        }
        Endpoint::OnRoad { road, at_m } => {
            hasher.write_u32(1);
            hasher.write_u32(road);
            hasher.write_f64(at_m);
        }
    }
}

fn hash_add_road_spec(spec: &AddRoadSpec, hasher: &mut Fnv64) {
    hash_endpoint(spec.from, hasher);
    hash_endpoint(spec.to, hasher);
    let via = spec.via.unwrap_or([f64::NAN; 2]);
    hasher.write_u32(u32::from(spec.via.is_some()));
    hasher.write_f64(via[0]);
    hasher.write_f64(via[1]);
    hasher.write_u32(u32::from(spec.lanes.0));
    hasher.write_u32(u32::from(spec.lanes.1));
    hasher.write_u32(spec.layer as u32);
}

fn hash_add_road_undo(undo: &AddRoadUndo, hasher: &mut Fnv64) {
    hasher.write_u32(11);
    hash_add_road_spec(&undo.spec, hasher);
    hasher.write_u32(undo.splits.len() as u32);
    for split in &undo.splits {
        hasher.write_u32(split.road);
        hasher.write_u32(split.r2);
        hasher.write_u32(split.old_range.0);
        hasher.write_u32(split.old_range.1);
        hasher.write_f64(split.old_length);
        hasher.write_u32(split.old_to);
        hasher.write_f64(split.cut);
    }
    hasher.write_u32(undo.road_len_before);
    hasher.write_u32(undo.node_len_before);
}

fn set_lanes(network: &mut Network, road: u32, forward: u8, backward: u8) -> Edit {
    let (forward, backward) = network.set_road_lanes(road, forward, backward);
    Edit::SetLanes {
        road,
        forward,
        backward,
    }
}

fn timing_words(key: u32, timing: Option<&Timing>) -> Vec<u32> {
    let mut words = vec![4, key];
    if let Some(timing) = timing {
        words.push(timing.offset_ticks);
        words.push(timing.green_ticks.len() as u32);
        words.extend_from_slice(&timing.green_ticks);
    }
    words
}

fn road_scope(network: &Network, road: u32, structural: bool) -> Scope {
    let index = road as usize;
    let mut nodes = vec![network.roads.from[index], network.roads.to[index]];
    nodes.sort_unstable();
    nodes.dedup();
    let signals = structural && nodes.iter().any(|&node| network.is_signal_node(node));
    Scope {
        roads: vec![road],
        invalidated: if structural {
            nodes.clone()
        } else {
            Vec::new()
        },
        nodes,
        signals,
    }
}

fn node_scope(node: u32, signals: bool) -> Scope {
    Scope {
        roads: Vec::new(),
        nodes: vec![node],
        invalidated: vec![node],
        signals,
    }
}
