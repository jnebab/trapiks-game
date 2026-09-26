use std::borrow::Cow;

use crate::network::{LinkId, Network};

use super::ahead::Ahead;
use super::approach::heads_to;
use super::lanes::landing_lane;
use super::pose::{movement_of, place_end};
use super::{Entry, Occupancy, Place, VehicleStore, link_key, place_key};

pub fn leader(
    network: &Network,
    vehicles: &VehicleStore,
    occupancy: &Occupancy,
    slot: u32,
) -> Option<(f64, f64)> {
    let own = own_lane_leader(network, vehicles, occupancy, slot);
    let serial = serial_leader(network, vehicles, occupancy, slot);
    match (own, serial) {
        (Some(a), Some(b)) if b.0 < a.0 => Some(b),
        (Some(a), _) => Some(a),
        (None, b) => b,
    }
}

fn own_lane_leader(
    network: &Network,
    vehicles: &VehicleStore,
    occupancy: &Occupancy,
    slot: u32,
) -> Option<(f64, f64)> {
    let index = slot as usize;
    let place = vehicles.place[index];
    let s = vehicles.s[index];
    if let Some(entry) = occupancy.same_place_leader(slot, place_key(place)) {
        return Some(gap_to(vehicles, s, entry.slot, entry.s));
    }
    let ahead = current_ahead(network, vehicles, slot);
    leader_beyond(vehicles, occupancy, &ahead, s, None)
}

pub fn leader_beyond(
    vehicles: &VehicleStore,
    occupancy: &Occupancy,
    ahead: &Ahead,
    s: f64,
    landing: Option<u64>,
) -> Option<(f64, f64)> {
    let mut acc = ahead.end()? - s;
    for (i, place) in ahead.places().iter().enumerate() {
        let key = match landing {
            Some(key) if i == 1 => key,
            _ => place.key,
        };
        if let Some((leader, leader_s)) = occupancy.last_on(key) {
            let gap = acc + leader_s - place.start - vehicles.length(leader);
            return Some((gap, vehicles.v[leader as usize]));
        }
        acc += place.end - place.start;
    }
    None
}

pub fn gap_to(vehicles: &VehicleStore, s: f64, leader: u32, leader_s: f64) -> (f64, f64) {
    let gap = leader_s - s - vehicles.length(leader);
    (gap, vehicles.v[leader as usize])
}

fn serial_leader(
    network: &Network,
    vehicles: &VehicleStore,
    occupancy: &Occupancy,
    slot: u32,
) -> Option<(f64, f64)> {
    let index = slot as usize;
    let Place::Link { link, lane } = vehicles.place[index] else {
        return None;
    };
    if !vehicles.committed[index] {
        return None;
    }
    let target = vehicles.route_link(slot, 1)?;
    let s = vehicles.s[index];
    (0..network.link_lanes(link))
        .filter(|&other| other != lane)
        .filter_map(|other| {
            nearest_serial(
                vehicles,
                occupancy.range(link_key(link, other)),
                (s, slot),
                target,
            )
        })
        .min_by(|a, b| a.s.total_cmp(&b.s).then_with(|| b.slot.cmp(&a.slot)))
        .map(|entry| gap_to(vehicles, s, entry.slot, entry.s))
}

fn nearest_serial(
    vehicles: &VehicleStore,
    range: &[Entry],
    (s, slot): (f64, u32),
    target: LinkId,
) -> Option<Entry> {
    let split = range.partition_point(|e| is_ahead(e.s, e.slot, s, slot));
    range[..split]
        .iter()
        .rev()
        .find(|e| vehicles.committed[e.slot as usize] && heads_to(vehicles, e.slot, target))
        .copied()
}

pub fn is_ahead(b_s: f64, b_slot: u32, a_s: f64, a_slot: u32) -> bool {
    b_s > a_s || (b_s == a_s && b_slot < a_slot)
}

pub fn current_ahead<'a>(
    network: &Network,
    vehicles: &'a VehicleStore,
    slot: u32,
) -> Cow<'a, Ahead> {
    let place = vehicles.place[slot as usize];
    let cursor = vehicles.cursor(slot);
    let cached = &vehicles.ahead[slot as usize];
    if cached.is_fresh(network, place, cursor) {
        return Cow::Borrowed(cached);
    }
    Cow::Owned(Ahead::compute(network, vehicles.route(slot), place, cursor))
}

pub fn extent(network: &Network, place: Place) -> Option<(f64, f64)> {
    match place {
        Place::Link { link, .. } => Some(network.link_span(link)),
        Place::Movement { .. } => Some((0.0, place_end(network, place)?)),
    }
}

pub fn next_place(
    network: &Network,
    route: &[LinkId],
    place: Place,
    cursor: usize,
) -> Option<(Place, usize)> {
    match place {
        Place::Link { link, lane } => {
            let next = *route.get(cursor + 1)?;
            let node = network.link_to(link);
            let index = network.junction(node)?.movement_index(link, next)?;
            let movement = movement_of(network, node, index as u16)?;
            let to_lane = landing_lane(network, movement, route, cursor + 1, lane)?;
            let entered = Place::Movement {
                node,
                movement: index as u16,
                from_lane: lane,
                to_lane,
            };
            Some((entered, cursor))
        }
        Place::Movement { to_lane, .. } => {
            let link = *route.get(cursor + 1)?;
            Some((
                Place::Link {
                    link,
                    lane: to_lane,
                },
                cursor + 1,
            ))
        }
    }
}
