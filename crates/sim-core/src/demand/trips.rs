use std::collections::VecDeque;

use crate::consts::{DESTINATION_DRAWS, TRIP_EXPIRY_TICKS};
use crate::network::{LinkId, Network, reverse};
use crate::rng::Pcg32;

use super::DemandTables;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trip {
    pub from: LinkId,
    pub to: LinkId,
    pub created_tick: u64,
}

impl Trip {
    pub fn is_expired(&self, tick: u64) -> bool {
        tick.saturating_sub(self.created_tick) >= TRIP_EXPIRY_TICKS
    }
}

#[derive(Clone, Debug, Default)]
pub struct TripQueue {
    pub trips: VecDeque<Trip>,
}

impl TripQueue {
    pub fn drop_expired(&mut self, tick: u64) -> u64 {
        let mut dropped = 0;
        while self.trips.front().is_some_and(|trip| trip.is_expired(tick)) {
            self.trips.pop_front();
            dropped += 1;
        }
        dropped
    }

    pub fn forget_links_from(&mut self, limit: LinkId) -> u64 {
        let before = self.trips.len();
        self.trips
            .retain(|trip| trip.from < limit && trip.to < limit);
        (before - self.trips.len()) as u64
    }
}

pub fn draw_trip(
    tables: &DemandTables,
    network: &Network,
    band: (f64, f64),
    tick: u64,
    rng: &mut Pcg32,
) -> Option<Trip> {
    let from = live_link(network, tables.origins.sample(rng)?)?;
    let start = network.nodes.pos[network.link_to(from) as usize];
    for _ in 0..DESTINATION_DRAWS {
        let to = live_link(network, tables.destinations.sample(rng)?)?;
        let end = network.nodes.pos[network.link_from(to) as usize];
        let distance = start.distance(end);
        if to != from && distance >= band.0 && distance <= band.1 {
            return Some(Trip {
                from,
                to,
                created_tick: tick,
            });
        }
    }
    None
}

fn live_link(network: &Network, link: LinkId) -> Option<LinkId> {
    [link, reverse(link)]
        .into_iter()
        .find(|&candidate| network.is_link_active(candidate))
}
