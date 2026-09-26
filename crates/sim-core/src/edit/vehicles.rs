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
        vehicles.place[index] = Place::Link {
            link,
            lane: lane.min(top),
        };
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
    vehicles.place[index] = Place::Movement {
        node: entry.node,
        movement: movement as u16,
        from_lane: from_lane.min(network.link_lanes(entry.from).saturating_sub(1)),
        to_lane: to_lane.min(network.link_lanes(entry.to).saturating_sub(1)),
    };
    vehicles.s[index] = vehicles.s[index].min(length);
    true
}

pub struct RouteDamage {
    dead_links: BTreeSet<LinkId>,
    invalidated: Vec<u32>,
    valid_turns: BTreeSet<(LinkId, LinkId)>,
}

impl RouteDamage {
    pub fn new(network: &Network, roads: &[u32], invalidated: &[u32]) -> RouteDamage {
        let dead_links = roads
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
            dead_links,
            invalidated: invalidated.to_vec(),
            valid_turns,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.dead_links.is_empty() && self.invalidated.is_empty()
    }

    pub fn breaks(&self, network: &Network, route: &[LinkId]) -> bool {
        route.iter().any(|link| self.dead_links.contains(link))
            || route
                .windows(2)
                .any(|pair| self.breaks_turn(network, pair[0], pair[1]))
    }

    fn breaks_turn(&self, network: &Network, from: LinkId, to: LinkId) -> bool {
        let node = network.link_to(from);
        self.invalidated.binary_search(&node).is_ok() && !self.valid_turns.contains(&(from, to))
    }
}

pub fn flag_routes(
    network: &Network,
    vehicles: &VehicleStore,
    damage: &RouteDamage,
    flagged: &mut BTreeSet<u32>,
) -> u64 {
    let mut marked = 0;
    for slot in vehicles.live_slots() {
        let route = vehicles.route(slot);
        let rest = route.get(vehicles.cursor(slot)..).unwrap_or_default();
        if damage.breaks(network, rest) && flagged.insert(slot) {
            marked += 1;
        }
    }
    marked
}
