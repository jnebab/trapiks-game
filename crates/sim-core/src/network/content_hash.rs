use crate::fnv::Fnv64;

use super::Network;

impl Network {
    pub fn content_hash(&self) -> u64 {
        if let Some((version, hash)) = self.content.get()
            && version == self.version
        {
            return hash;
        }
        let hash = self.compute_content_hash();
        self.content.set(Some((self.version, hash)));
        hash
    }

    fn compute_content_hash(&self) -> u64 {
        let mut hasher = Fnv64::new();
        for road in 0..self.roads.count() {
            self.hash_road(&mut hasher, road);
        }
        for control in &self.nodes.control {
            hasher.write(&[control.code()]);
        }
        for ban in &self.bans {
            hasher.write_u32(ban.via_node);
            hasher.write_u32(ban.from_road);
            hasher.write_u32(ban.to_road);
        }
        for (&key, timing) in &self.timings {
            hasher.write_u32(key);
            hasher.write_u32(timing.offset_ticks);
            hasher.write_u32(timing.green_ticks.len() as u32);
            for &green in &timing.green_ticks {
                hasher.write_u32(green);
            }
        }
        hasher.finish()
    }

    fn hash_road(&self, hasher: &mut Fnv64, road: usize) {
        let roads = &self.roads;
        hasher.write(&[
            u8::from(roads.deleted[road]),
            roads.lanes_forward[road],
            roads.lanes_backward[road],
            roads.speed_kph[road],
            roads.layer[road] as u8,
        ]);
        hasher.write_u32(roads.from[road]);
        hasher.write_u32(roads.to[road]);
    }
}
