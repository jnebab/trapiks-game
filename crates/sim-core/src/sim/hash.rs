use crate::fnv::Fnv64;
use crate::vehicle::Place;

use super::Sim;

impl Sim {
    pub fn state_hash(&self) -> u64 {
        let mut hasher = Fnv64::new();
        let (state, inc) = self.rng.state_words();
        hasher.write_u64(self.tick);
        hasher.write_u64(state);
        hasher.write_u64(inc);
        hasher.write_u64(self.network.version());
        hasher.write_u32(self.vehicles.live());
        for slot in self.vehicles.live_slots() {
            self.hash_vehicle(&mut hasher, slot);
        }
        self.hash_demand(&mut hasher);
        self.hash_edits(&mut hasher);
        for speed in &self.costs.ema_speed {
            hasher.write_u64(speed.to_bits());
        }
        hasher.finish()
    }

    fn hash_edits(&self, hasher: &mut Fnv64) {
        hasher.write_u64(self.edits.undo.len() as u64);
        for entry in &self.edits.undo {
            entry.inverse.hash_into(hasher);
            hasher.write_u64(entry.cost as u64);
        }
        hasher.write_u64(self.edits.budget.spent as u64);
        hasher.write_u64(self.network.content_hash());
        for &slot in &self.edits.flagged {
            hasher.write_u32(slot);
        }
        hasher.write_u64(self.edits.commands.len() as u64);
    }

    fn hash_demand(&self, hasher: &mut Fnv64) {
        hasher.write_u64(self.demand.clock.next_s().to_bits());
        let queued = self.demand.trips.trips.iter();
        for trip in queued.chain(self.demand.queues.trips()) {
            hasher.write_u32(trip.from);
            hasher.write_u32(trip.to);
            hasher.write_u64(trip.created_tick);
            hasher.write_u32(u32::from(trip.kind.code()));
        }
    }

    fn hash_vehicle(&self, hasher: &mut Fnv64, slot: u32) {
        let index = slot as usize;
        let vehicles = &self.vehicles;
        hasher.write_u32(slot);
        hasher.write_u32(vehicles.id[index]);
        hash_place(hasher, vehicles.place[index]);
        hasher.write_u64(vehicles.s[index].to_bits());
        hasher.write_u64(vehicles.v[index].to_bits());
        hasher.write_u32(u32::from(vehicles.route_cursor[index]));
        hasher.write_u32(u32::from(vehicles.committed[index]));
        hasher.write_u64(vehicles.arrival_tick[index]);
        hasher.write_u32(vehicles.wait_ticks[index]);
        hasher.write_u32(u32::from(vehicles.stopped_at_line[index]));
        hasher.write_u32(u32::from(vehicles.kind[index].code()));
        hasher.write_u32(u32::from(vehicles.lane_from[index]));
        hasher.write_u64(vehicles.lane_change_tick[index]);
        for &link in vehicles.route(slot) {
            hasher.write_u32(link);
        }
    }
}

fn hash_place(hasher: &mut Fnv64, place: Place) {
    let words = match place {
        Place::Link { link, lane } => [0, link, u32::from(lane), 0, 0],
        Place::Movement {
            node,
            movement,
            from_lane,
            to_lane,
        } => [
            1,
            node,
            u32::from(movement),
            u32::from(from_lane),
            u32::from(to_lane),
        ],
    };
    for word in words {
        hasher.write_u32(word);
    }
}
