use crate::network::LinkId;

use super::{Place, SpawnError};

const MAX_ROUTE_LEN: usize = u16::MAX as usize;

pub struct NewVehicle<'a> {
    pub place: Place,
    pub s: f64,
    pub v: f64,
    pub color: u8,
    pub route: &'a [LinkId],
    pub tick: u64,
}

pub struct VehicleStore {
    pub alive: Vec<bool>,
    pub id: Vec<u32>,
    pub place: Vec<Place>,
    pub s: Vec<f64>,
    pub v: Vec<f64>,
    pub color: Vec<u8>,
    pub route_start: Vec<u32>,
    pub route_len: Vec<u16>,
    pub route_cursor: Vec<u16>,
    pub spawn_tick: Vec<u64>,
    pub routes: Vec<LinkId>,
    free: Vec<u32>,
    scratch: Vec<LinkId>,
    capacity: usize,
    dead_len: usize,
    next_id: u32,
    live: u32,
}

impl VehicleStore {
    pub fn with_capacity(capacity: usize) -> Self {
        VehicleStore {
            alive: Vec::with_capacity(capacity),
            id: Vec::with_capacity(capacity),
            place: Vec::with_capacity(capacity),
            s: Vec::with_capacity(capacity),
            v: Vec::with_capacity(capacity),
            color: Vec::with_capacity(capacity),
            route_start: Vec::with_capacity(capacity),
            route_len: Vec::with_capacity(capacity),
            route_cursor: Vec::with_capacity(capacity),
            spawn_tick: Vec::with_capacity(capacity),
            routes: Vec::new(),
            free: Vec::with_capacity(capacity),
            scratch: Vec::new(),
            capacity,
            dead_len: 0,
            next_id: 0,
            live: 0,
        }
    }

    pub fn check_route_len(route: &[LinkId]) -> Result<u16, SpawnError> {
        if route.len() > MAX_ROUTE_LEN {
            return Err(SpawnError::TooLong);
        }
        Ok(route.len() as u16)
    }

    pub fn is_full(&self) -> bool {
        self.live as usize >= self.capacity
    }

    pub fn live(&self) -> u32 {
        self.live
    }

    pub fn slot_count(&self) -> usize {
        self.alive.len()
    }

    pub fn live_slots(&self) -> impl Iterator<Item = u32> + '_ {
        self.alive
            .iter()
            .enumerate()
            .filter(|(_, alive)| **alive)
            .map(|(slot, _)| slot as u32)
    }

    pub fn route(&self, slot: u32) -> &[LinkId] {
        let index = slot as usize;
        let start = self.route_start[index] as usize;
        let len = usize::from(self.route_len[index]);
        &self.routes[start..start + len]
    }

    pub fn cursor(&self, slot: u32) -> usize {
        usize::from(self.route_cursor[slot as usize])
    }

    pub fn route_link(&self, slot: u32, offset: usize) -> Option<LinkId> {
        self.route(slot).get(self.cursor(slot) + offset).copied()
    }

    pub fn insert(&mut self, vehicle: &NewVehicle) -> Result<(u32, u32), SpawnError> {
        let len = Self::check_route_len(vehicle.route)?;
        if self.is_full() {
            return Err(SpawnError::Full);
        }
        let id = self.next_id;
        self.next_id += 1;
        self.live += 1;
        let start = self.routes.len() as u32;
        self.routes.extend_from_slice(vehicle.route);
        let slot = self.allocate();
        let index = slot as usize;
        self.alive[index] = true;
        self.id[index] = id;
        self.place[index] = vehicle.place;
        self.s[index] = vehicle.s;
        self.v[index] = vehicle.v;
        self.color[index] = vehicle.color;
        self.route_start[index] = start;
        self.route_len[index] = len;
        self.route_cursor[index] = 0;
        self.spawn_tick[index] = vehicle.tick;
        Ok((slot, id))
    }

    fn allocate(&mut self) -> u32 {
        if let Some(slot) = self.free.pop() {
            return slot;
        }
        let slot = self.alive.len() as u32;
        self.alive.push(false);
        self.id.push(0);
        self.place.push(Place::Link { link: 0, lane: 0 });
        self.s.push(0.0);
        self.v.push(0.0);
        self.color.push(0);
        self.route_start.push(0);
        self.route_len.push(0);
        self.route_cursor.push(0);
        self.spawn_tick.push(0);
        slot
    }

    pub fn release(&mut self, slot: u32) {
        let index = slot as usize;
        if !self.alive[index] {
            return;
        }
        self.alive[index] = false;
        self.live -= 1;
        self.dead_len += usize::from(self.route_len[index]);
        self.free.push(slot);
    }

    pub fn compact_routes(&mut self) {
        if self.dead_len * 2 <= self.routes.len() {
            return;
        }
        self.scratch.clear();
        for index in 0..self.alive.len() {
            if !self.alive[index] {
                continue;
            }
            let start = self.route_start[index] as usize;
            let len = usize::from(self.route_len[index]);
            self.route_start[index] = self.scratch.len() as u32;
            self.scratch
                .extend_from_slice(&self.routes[start..start + len]);
        }
        std::mem::swap(&mut self.routes, &mut self.scratch);
        self.dead_len = 0;
    }
}
