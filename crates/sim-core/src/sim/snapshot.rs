use crate::geom::Vec2;
use crate::network::{LinkId, road_of};
use crate::vehicle::lane_change::LC_ANIMATION_TICKS;
use crate::vehicle::{Place, link_pose_blended, pose};

use super::Sim;

#[derive(Debug, Default)]
pub struct Snapshot {
    pub ids: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub heading: Vec<f32>,
    pub style: Vec<u8>,
    pub layer: Vec<i8>,
}

impl Snapshot {
    fn clear(&mut self) {
        self.ids.clear();
        self.x.clear();
        self.y.clear();
        self.heading.clear();
        self.style.clear();
        self.layer.clear();
    }
}

impl Sim {
    pub fn fill_snapshot(&self, out: &mut Snapshot) {
        out.clear();
        for slot in self.vehicles.live_slots() {
            let index = slot as usize;
            let (pos, heading) = self.vehicle_pose(slot);
            out.ids.push(self.vehicles.id[index]);
            out.x.push(pos.x as f32);
            out.y.push(pos.y as f32);
            out.heading.push(heading as f32);
            out.style
                .push(self.vehicles.kind[index].code() * 16 + self.vehicles.color[index]);
            out.layer.push(self.place_layer(self.vehicles.place[index]));
        }
    }

    fn place_layer(&self, place: Place) -> i8 {
        match place {
            Place::Link { link, .. } => self.link_layer(link),
            Place::Movement { node, movement, .. } => self.movement_layer(node, movement),
        }
    }

    fn movement_layer(&self, node: u32, movement: u16) -> i8 {
        let Some(found) = self
            .network
            .junction(node)
            .and_then(|junction| junction.movements.get(movement as usize))
        else {
            return 0;
        };
        self.link_layer(found.from_link)
            .max(self.link_layer(found.to_link))
    }

    fn link_layer(&self, link: LinkId) -> i8 {
        self.network
            .roads
            .layer
            .get(road_of(link) as usize)
            .copied()
            .unwrap_or(0)
    }

    fn vehicle_pose(&self, slot: u32) -> (Vec2, f64) {
        let index = slot as usize;
        let place = self.vehicles.place[index];
        let s = self.vehicles.s[index];
        let lane_from = self.vehicles.lane_from[index];
        match place {
            Place::Link { link, lane } if lane != lane_from => {
                let since = self
                    .tick
                    .saturating_sub(self.vehicles.lane_change_tick[index]);
                let u = since as f64 / LC_ANIMATION_TICKS as f64;
                link_pose_blended(&self.network, link, s, (lane_from, lane), u)
            }
            _ => pose(&self.network, place, s).unwrap_or_default(),
        }
    }
}
