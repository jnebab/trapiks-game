use std::collections::BTreeSet;

use crate::network::{LinkId, Network};
use crate::vehicle::{Place, VehicleStore};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeldMovement {
    pub slot: u32,
    pub node: u32,
    pub from: LinkId,
    pub to: LinkId,
}

pub fn record_movements(
    network: &Network,
    vehicles: &VehicleStore,
    invalidated: &[u32],
) -> Vec<HeldMovement> {
    vehicles
        .live_slots()
        .filter_map(|slot| held_movement(network, vehicles, invalidated, slot))
        .collect()
}

fn held_movement(
    network: &Network,
    vehicles: &VehicleStore,
    invalidated: &[u32],
    slot: u32,
) -> Option<HeldMovement> {
    let Place::Movement { node, movement, .. } = vehicles.place[slot as usize] else {
        return None;
    };
    invalidated.binary_search(&node).ok()?;
    let movement = network
        .junction(node)?
        .movements
        .get(usize::from(movement))?;
    Some(HeldMovement {
        slot,
        node,
        from: movement.from_link,
        to: movement.to_link,
    })
}

pub fn settle_links(
    network: &Network,
    vehicles: &mut VehicleStore,
    invalidated: &[u32],
) -> Vec<u32> {
    let mut stranded = Vec::new();
    for slot in 0..vehicles.slot_count() as u32 {
        let index = slot as usize;
        if !vehicles.alive[index] {
            continue;
        }
        let Place::Link { link, lane } = vehicles.place[index] else {
            continue;
        };
        if !network.is_link_active(link) {
            stranded.push(slot);
            continue;
        }
        let top = network.link_lanes(link) - 1;
        vehicles.set_place(
            slot,
            Place::Link {
                link,
                lane: lane.min(top),
            },
        );
        if invalidated.binary_search(&network.link_to(link)).is_ok() {
            vehicles.committed[index] = false;
        }
    }
    stranded
}

pub fn remap_movements(
    network: &mut Network,
    vehicles: &mut VehicleStore,
    held: &[HeldMovement],
) -> Vec<u32> {
    held.iter()
        .filter(|entry| !remap_one(network, vehicles, entry))
        .map(|entry| entry.slot)
        .collect()
}

fn remap_one(network: &mut Network, vehicles: &mut VehicleStore, entry: &HeldMovement) -> bool {
    let index = entry.slot as usize;
    let Place::Movement {
        from_lane, to_lane, ..
    } = vehicles.place[index]
    else {
        return false;
    };
    let junction = network.ensure_junction(entry.node);
    let Some(movement) = junction.movement_index(entry.from, entry.to) else {
        return false;
    };
    let length = junction.movements[movement].length;
    let place = Place::Movement {
        node: entry.node,
        movement: movement as u16,
        from_lane: from_lane.min(network.link_lanes(entry.from).saturating_sub(1)),
        to_lane: to_lane.min(network.link_lanes(entry.to).saturating_sub(1)),
    };
    vehicles.set_place(entry.slot, place);
    vehicles.s[index] = vehicles.s[index].min(length);
    true
}

pub struct RouteDamage {
    dead_links: Vec<bool>,
    into_invalidated: Vec<bool>,
    empty: bool,
    valid_turns: BTreeSet<(LinkId, LinkId)>,
}

impl RouteDamage {
    pub fn new(network: &Network, roads: &[u32], invalidated: &[u32]) -> RouteDamage {
        let dead: Vec<LinkId> = roads
            .iter()
            .flat_map(|&road| [road * 2, road * 2 + 1])
            .filter(|&link| !network.is_link_active(link))
            .collect();
        let valid_turns = invalidated
            .iter()
            .flat_map(|&node| network.turns(node))
            .map(|(from, to, _)| (from, to))
            .collect();
        RouteDamage {
            dead_links: mask(network.link_count(), &dead),
            into_invalidated: mask(network.link_count(), &arriving_links(network, invalidated)),
            empty: dead.is_empty() && invalidated.is_empty(),
            valid_turns,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn breaks(&self, route: &[LinkId]) -> bool {
        route.iter().any(|&link| flagged(&self.dead_links, link))
            || route
                .windows(2)
                .any(|pair| self.breaks_turn(pair[0], pair[1]))
    }

    fn breaks_turn(&self, from: LinkId, to: LinkId) -> bool {
        flagged(&self.into_invalidated, from) && !self.valid_turns.contains(&(from, to))
    }
}

fn arriving_links(network: &Network, nodes: &[u32]) -> Vec<LinkId> {
    nodes
        .iter()
        .flat_map(|&node| {
            network.nodes.roads[node as usize]
                .iter()
                .flat_map(|&road| [road * 2, road * 2 + 1])
                .filter(move |&link| network.link_to(link) == node)
        })
        .collect()
}

fn mask(len: usize, set: &[u32]) -> Vec<bool> {
    let mut mask = vec![false; len];
    for &index in set {
        if let Some(flag) = mask.get_mut(index as usize) {
            *flag = true;
        }
    }
    mask
}

fn flagged(mask: &[bool], index: u32) -> bool {
    mask.get(index as usize).copied().unwrap_or(false)
}

pub fn flag_routes(
    vehicles: &VehicleStore,
    damage: &RouteDamage,
    flagged: &mut BTreeSet<u32>,
) -> u64 {
    let mut marked = 0;
    for slot in vehicles.live_slots() {
        let route = vehicles.route(slot);
        let rest = route.get(vehicles.cursor(slot)..).unwrap_or_default();
        if damage.breaks(rest) && flagged.insert(slot) {
            marked += 1;
        }
    }
    marked
}
