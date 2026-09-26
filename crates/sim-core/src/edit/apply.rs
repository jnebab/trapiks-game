use crate::fnv::Fnv64;
use crate::map::{Control, TurnBan};
use crate::network::{Network, Timing};

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
        }
    }

    pub fn hash_into(&self, hasher: &mut Fnv64) {
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
        };
        for word in words {
            hasher.write_u32(word);
        }
    }
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
