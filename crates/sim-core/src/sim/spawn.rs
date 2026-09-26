use crate::consts::{COLOR_COUNT, IDM_MIN_GAP};
use crate::network::LinkId;
use crate::vehicle::lanes::allowed_lanes;
use crate::vehicle::space::free_space;
use crate::vehicle::{
    NewVehicle, Place, SpawnError, VehicleKind, VehicleStore, entry_of, movement_key, place_key,
};

use super::Sim;

const LEADER_SPEED_RANGE: f64 = 50.0;

impl Sim {
    pub fn spawn(&mut self, route: &[LinkId]) -> Result<u32, SpawnError> {
        self.spawn_kind(route, VehicleKind::Car)
    }

    pub fn spawn_kind(&mut self, route: &[LinkId], kind: VehicleKind) -> Result<u32, SpawnError> {
        let first = *route.first().ok_or(SpawnError::EmptyRoute)?;
        let len = VehicleStore::check_route_len(route)?;
        self.check_active(route)?;
        self.check_connected(route)?;
        if self.vehicles.is_full() {
            return Err(SpawnError::Full);
        }
        let lane = self.first_lane(route);
        let start = self.network.link_span(first).0;
        if self.landing_onto(first, lane) {
            return Err(SpawnError::Blocked);
        }
        let place = Place::Link { link: first, lane };
        let v = self.entry_speed(first, place, kind)?;
        let color = self.rng.below(COLOR_COUNT) as u8;
        let vehicle = NewVehicle {
            place,
            s: start,
            v,
            color,
            kind,
            route,
            tick: self.tick,
        };
        let (slot, id) = self.vehicles.insert(&vehicle, len);
        self.vehicles.free_flow[slot as usize] = self.route_free_time(route) / kind.speed_factor();
        self.occupancy.push_pending(entry_of(&self.vehicles, slot));
        self.stats.spawned += 1;
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
        let Some((low, high)) = allowed_lanes(&self.network, route, 0) else {
            return 0;
        };
        let first = route[0];
        let mut best = (low, f64::NEG_INFINITY);
        for lane in low..=high {
            let free = free_space(&self.network, &self.vehicles, &self.occupancy, first, lane);
            if free.length > best.1 {
                best = (lane, free.length);
            }
        }
        best.0
    }

    fn landing_onto(&self, link: LinkId, lane: u8) -> bool {
        let node = self.network.link_from(link);
        let Some(junction) = self.network.junction(node) else {
            return false;
        };
        junction
            .movements
            .iter()
            .enumerate()
            .filter(|(_, m)| m.to_link == link)
            .any(|(index, _)| self.lands_via(node, index as u16, lane))
    }

    fn lands_via(&self, node: u32, movement: u16, lane: u8) -> bool {
        self.occupancy
            .slots_on(movement_key(node, movement))
            .any(|slot| self.is_landing(slot, node, movement, lane))
    }

    fn is_landing(&self, slot: u32, node: u32, movement: u16, lane: u8) -> bool {
        let index = slot as usize;
        self.vehicles.alive[index]
            && matches!(
                self.vehicles.place[index],
                Place::Movement { node: n, movement: m, to_lane, .. }
                    if n == node && m == movement && to_lane == lane
            )
    }

    fn entry_speed(
        &self,
        link: LinkId,
        place: Place,
        kind: VehicleKind,
    ) -> Result<f64, SpawnError> {
        let v0 = self.network.link_speed(link) * kind.speed_factor();
        let Some((leader, leader_s)) = self.occupancy.last_on(place_key(place)) else {
            return Ok(v0);
        };
        let start = self.network.link_span(link).0;
        let distance = leader_s - start;
        if distance - self.vehicles.length(leader) < IDM_MIN_GAP {
            return Err(SpawnError::Blocked);
        }
        if distance > LEADER_SPEED_RANGE {
            return Ok(v0);
        }
        Ok(v0.min(self.vehicles.v[leader as usize]))
    }
}
