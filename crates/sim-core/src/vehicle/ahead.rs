use crate::consts::LEADER_LOOKAHEAD_PLACES;
use crate::network::{LinkId, Network};

use super::leader::{extent, next_place};
use super::pose::place_end;
use super::{Place, place_key};

const STALE: u64 = u64::MAX;
const MAX_NODES: usize = 2 * (LEADER_LOOKAHEAD_PLACES + 1);

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
    from_lanes: Option<(u8, u8)>,
    landing: Option<u8>,
    len: usize,
    places: [AheadPlace; LEADER_LOOKAHEAD_PLACES],
    node_len: usize,
    nodes: [u32; MAX_NODES],
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
        from_lanes: None,
        landing: None,
        len: 0,
        places: [AheadPlace {
            key: 0,
            start: 0.0,
            end: 0.0,
        }; LEADER_LOOKAHEAD_PLACES],
        node_len: 0,
        nodes: [0; MAX_NODES],
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
        ahead.record_nodes(network, place);
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
            ahead.record_nodes(network, next);
            (at, at_cursor) = (next, next_cursor);
        }
        ahead.movement = relevant_movement(network, place, ahead.next);
        ahead.crossing = has_crossing(network, ahead.movement);
        ahead.from_lanes = from_lanes(network, place, ahead.movement);
        ahead
    }

    pub fn movement(&self) -> Option<(u32, u16)> {
        self.movement
    }

    pub fn from_lanes(&self) -> Option<(u8, u8)> {
        self.from_lanes
    }

    pub fn landing(&self) -> Option<u8> {
        self.landing
    }

    pub fn crossing(&self) -> Option<(u32, u16)> {
        self.movement.filter(|_| self.crossing)
    }

    fn record_nodes(&mut self, network: &Network, place: Place) {
        let nodes = match place {
            Place::Link { link, .. } => [network.link_from(link), network.link_to(link)],
            Place::Movement { node, .. } => [node, node],
        };
        for node in nodes {
            self.record_node(node);
        }
    }

    fn record_node(&mut self, node: u32) {
        if self.node_len > 0 && self.nodes[self.node_len - 1] == node {
            return;
        }
        if let Some(slot) = self.nodes.get_mut(self.node_len) {
            *slot = node;
            self.node_len += 1;
        }
    }

    pub fn is_fresh(&self, network: &Network, place: Place, cursor: usize) -> bool {
        self.place == place && self.cursor == cursor && self.is_current(network)
    }

    fn is_current(&self, network: &Network) -> bool {
        if self.version == network.version() {
            return true;
        }
        self.version != STALE
            && self.version >= network.global_version()
            && self.nodes[..self.node_len]
                .iter()
                .all(|&node| network.node_version(node) <= self.version)
    }

    pub fn restamp(&mut self, network: &Network) {
        self.version = network.version();
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

fn from_lanes(network: &Network, place: Place, movement: Option<(u32, u16)>) -> Option<(u8, u8)> {
    let Place::Link { .. } = place else {
        return None;
    };
    let (node, index) = movement?;
    let movement = network.junction(node)?.movements.get(usize::from(index))?;
    Some(movement.from_lanes)
}

fn landing_of(next: Place) -> Option<u8> {
    match next {
        Place::Movement { to_lane, .. } => Some(to_lane),
        Place::Link { .. } => None,
    }
}
