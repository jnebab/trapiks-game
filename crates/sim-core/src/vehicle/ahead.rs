use crate::consts::LEADER_LOOKAHEAD_PLACES;
use crate::network::{LinkId, Network};

use super::leader::{extent, next_place};
use super::pose::place_end;
use super::{Place, place_key};

const STALE: u64 = u64::MAX;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AheadPlace {
    pub key: u64,
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ahead {
    place: Place,
    cursor: usize,
    version: u64,
    end: Option<f64>,
    next: Option<LinkId>,
    movement: Option<(u32, u16)>,
    crossing: bool,
    landing: Option<u8>,
    len: usize,
    places: [AheadPlace; LEADER_LOOKAHEAD_PLACES],
}

impl Ahead {
    pub const STALE: Ahead = Ahead {
        place: Place::Link { link: 0, lane: 0 },
        cursor: 0,
        version: STALE,
        end: None,
        next: None,
        movement: None,
        crossing: false,
        landing: None,
        len: 0,
        places: [AheadPlace {
            key: 0,
            start: 0.0,
            end: 0.0,
        }; LEADER_LOOKAHEAD_PLACES],
    };

    pub fn compute(network: &Network, route: &[LinkId], place: Place, cursor: usize) -> Ahead {
        let mut ahead = Ahead {
            place,
            cursor,
            version: network.version(),
            end: place_end(network, place),
            next: route.get(cursor + 1).copied(),
            ..Ahead::STALE
        };
        let (mut at, mut at_cursor) = (place, cursor);
        while ahead.len < LEADER_LOOKAHEAD_PLACES {
            let Some((next, next_cursor)) = next_place(network, route, at, at_cursor) else {
                break;
            };
            let Some((start, end)) = extent(network, next) else {
                break;
            };
            if ahead.len == 0 {
                ahead.landing = landing_of(next);
            }
            ahead.places[ahead.len] = AheadPlace {
                key: place_key(next),
                start,
                end,
            };
            ahead.len += 1;
            (at, at_cursor) = (next, next_cursor);
        }
        ahead.movement = relevant_movement(network, place, ahead.next);
        ahead.crossing = has_crossing(network, ahead.movement);
        ahead
    }

    pub fn movement(&self) -> Option<(u32, u16)> {
        self.movement
    }

    pub fn landing(&self) -> Option<u8> {
        self.landing
    }

    pub fn crossing(&self) -> Option<(u32, u16)> {
        self.movement.filter(|_| self.crossing)
    }

    pub fn is_fresh(&self, network: &Network, place: Place, cursor: usize) -> bool {
        self.version == network.version() && self.place == place && self.cursor == cursor
    }

    pub fn end(&self) -> Option<f64> {
        self.end
    }

    pub fn next(&self) -> Option<LinkId> {
        self.next
    }

    pub fn places(&self) -> &[AheadPlace] {
        &self.places[..self.len]
    }
}

fn relevant_movement(network: &Network, place: Place, next: Option<LinkId>) -> Option<(u32, u16)> {
    match place {
        Place::Movement { node, movement, .. } => Some((node, movement)),
        Place::Link { link, .. } => {
            let node = network.link_to(link);
            let index = network.junction(node)?.movement_index(link, next?)?;
            Some((node, u16::try_from(index).ok()?))
        }
    }
}

fn has_crossing(network: &Network, movement: Option<(u32, u16)>) -> bool {
    let Some((node, index)) = movement else {
        return false;
    };
    network
        .junction(node)
        .and_then(|junction| junction.has_crossing.get(usize::from(index)))
        .is_some_and(|&crossing| crossing)
}

fn landing_of(next: Place) -> Option<u8> {
    match next {
        Place::Movement { to_lane, .. } => Some(to_lane),
        Place::Link { .. } => None,
    }
}
