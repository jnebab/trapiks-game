use crate::consts::LEADER_LOOKAHEAD_PLACES;
use crate::network::{LinkId, Network};

use super::leader::{extent, next_place};
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
    len: usize,
    places: [AheadPlace; LEADER_LOOKAHEAD_PLACES],
}

impl Ahead {
    pub const STALE: Ahead = Ahead {
        place: Place::Link { link: 0, lane: 0 },
        cursor: 0,
        version: STALE,
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
            ahead.places[ahead.len] = AheadPlace {
                key: place_key(next),
                start,
                end,
            };
            ahead.len += 1;
            (at, at_cursor) = (next, next_cursor);
        }
        ahead
    }

    pub fn is_fresh(&self, network: &Network, place: Place, cursor: usize) -> bool {
        self.version == network.version() && self.place == place && self.cursor == cursor
    }

    pub fn places(&self) -> &[AheadPlace] {
        &self.places[..self.len]
    }
}
