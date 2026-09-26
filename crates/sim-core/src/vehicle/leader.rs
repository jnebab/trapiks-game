use crate::consts::CAR_LENGTH;
use crate::network::{LinkId, Network};

use super::ahead::Ahead;
use super::lanes::landing_lane;
use super::pose::{movement_of, place_end};
use super::{Occupancy, Place, VehicleStore, place_key};

pub fn leader(
    network: &Network,
    vehicles: &VehicleStore,
    occupancy: &Occupancy,
    slot: u32,
) -> Option<(f64, f64)> {
    let index = slot as usize;
    let place = vehicles.place[index];
    let s = vehicles.s[index];
    if let Some(entry) = occupancy.same_place_leader(slot, place_key(place)) {
        return Some((entry.s - s - CAR_LENGTH, vehicles.v[entry.slot as usize]));
    }
    let acc = place_end(network, place)? - s;
    let view = View {
        network,
        vehicles,
        occupancy,
    };
    look_ahead(&view, slot, place, acc)
}

struct View<'a> {
    network: &'a Network,
    vehicles: &'a VehicleStore,
    occupancy: &'a Occupancy,
}

fn look_ahead(view: &View, slot: u32, start: Place, mut acc: f64) -> Option<(f64, f64)> {
    let (network, vehicles) = (view.network, view.vehicles);
    let cursor = vehicles.cursor(slot);
    let cached = vehicles.ahead[slot as usize];
    let ahead = if cached.is_fresh(network, start, cursor) {
        cached
    } else {
        Ahead::compute(network, vehicles.route(slot), start, cursor)
    };
    for place in ahead.places() {
        if let Some((leader, leader_s)) = view.occupancy.last_on(place.key) {
            let gap = acc + leader_s - place.start - CAR_LENGTH;
            return Some((gap, vehicles.v[leader as usize]));
        }
        acc += place.end - place.start;
    }
    None
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
            let to_lane = landing_lane(network, movement, route, cursor + 1)?;
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
