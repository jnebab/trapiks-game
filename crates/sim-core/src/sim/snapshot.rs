use crate::vehicle::pose;

use super::Sim;

#[derive(Debug, Default)]
pub struct Snapshot {
    pub ids: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub heading: Vec<f32>,
    pub style: Vec<u8>,
}

impl Snapshot {
    fn clear(&mut self) {
        self.ids.clear();
        self.x.clear();
        self.y.clear();
        self.heading.clear();
        self.style.clear();
    }
}

impl Sim {
    pub fn fill_snapshot(&self, out: &mut Snapshot) {
        out.clear();
        for slot in self.vehicles.live_slots() {
            let index = slot as usize;
            let (pos, heading) = pose(
                &self.network,
                self.vehicles.place[index],
                self.vehicles.s[index],
            )
            .unwrap_or_default();
            out.ids.push(self.vehicles.id[index]);
            out.x.push(pos.x as f32);
            out.y.push(pos.y as f32);
            out.heading.push(heading as f32);
            out.style.push(self.vehicles.color[index]);
        }
    }
}
