use crate::consts::{CAR_LENGTH, COLOR_COUNT, IDM_MIN_GAP};
use crate::network::LinkId;
use crate::vehicle::lanes::entry_lane;
use crate::vehicle::{NewVehicle, Place, SpawnError, VehicleStore, entry_of, place_key};

use super::Sim;

const LEADER_SPEED_RANGE: f64 = 50.0;

impl Sim {
    pub fn spawn(&mut self, route: &[LinkId]) -> Result<u32, SpawnError> {
        let first = *route.first().ok_or(SpawnError::EmptyRoute)?;
        let len = VehicleStore::check_route_len(route)?;
        self.check_active(route)?;
        self.check_connected(route)?;
        if self.vehicles.is_full() {
            return Err(SpawnError::Full);
        }
        let lane = self.first_lane(route);
        let start = self.network.link_span(first).0;
        let place = Place::Link { link: first, lane };
        let v = self.entry_speed(first, place, start)?;
        let color = self.rng.below(COLOR_COUNT) as u8;
        let vehicle = NewVehicle {
            place,
            s: start,
            v,
            color,
            route,
            tick: self.tick,
        };
        let (slot, id) = self.vehicles.insert(&vehicle, len);
        self.occupancy.push_pending(entry_of(&self.vehicles, slot));
        Ok(id)
    }

    fn check_active(&self, route: &[LinkId]) -> Result<(), SpawnError> {
        match route
            .iter()
            .find(|&&link| !self.network.is_link_active(link))
        {
            Some(&link) => Err(SpawnError::InactiveLink(link)),
            None => Ok(()),
        }
    }

    fn check_connected(&mut self, route: &[LinkId]) -> Result<(), SpawnError> {
        for (i, pair) in route.windows(2).enumerate() {
            let node = self.network.link_to(pair[0]);
            let junction = self.network.ensure_junction(node);
            if junction.movement_index(pair[0], pair[1]).is_none() {
                return Err(SpawnError::Disconnected(i));
            }
        }
        Ok(())
    }

    fn first_lane(&self, route: &[LinkId]) -> u8 {
        let [first, second, ..] = *route else {
            return 0;
        };
        self.network
            .junction(self.network.link_to(first))
            .and_then(|junction| entry_lane(junction, first, second))
            .unwrap_or(0)
    }

    fn entry_speed(&self, link: LinkId, place: Place, start: f64) -> Result<f64, SpawnError> {
        let v0 = self.network.link_speed(link);
        let Some((leader, leader_s)) = self.occupancy.last_on(place_key(place)) else {
            return Ok(v0);
        };
        let distance = leader_s - start;
        if distance < IDM_MIN_GAP + CAR_LENGTH {
            return Err(SpawnError::Blocked);
        }
        if distance > LEADER_SPEED_RANGE {
            return Ok(v0);
        }
        Ok(v0.min(self.vehicles.v[leader as usize]))
    }
}
