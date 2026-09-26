use crate::consts::{DT, MIN_LINK_SPEED, STOPPED_SPEED};
use crate::network::road_of;
use crate::stats::{LiveSpeeds, StatsSnapshot};
use crate::vehicle::Place;

use super::Sim;

impl Sim {
    pub fn stats(&self) -> StatsSnapshot {
        self.stats.snapshot(self.tick, self.live_speeds())
    }

    pub fn reset_stats(&mut self) {
        self.stats.reset(self.tick);
    }

    pub fn road_speed_ratio(&self, out: &mut Vec<f32>) {
        let road_count = self.network.roads.count();
        let mut sums = vec![0.0f64; road_count];
        let mut counts = vec![0u32; road_count];
        for slot in self.vehicles.live_slots() {
            let index = slot as usize;
            let Place::Link { link, .. } = self.vehicles.place[index] else {
                continue;
            };
            let road = road_of(link) as usize;
            sums[road] += self.vehicles.v[index] / self.free_speed(link);
            counts[road] += 1;
        }
        out.clear();
        out.extend(sums.iter().zip(&counts).map(|(&sum, &count)| {
            if count == 0 {
                1.0
            } else {
                (sum / f64::from(count)) as f32
            }
        }));
    }

    pub(super) fn record_arrival(&mut self, slot: u32) {
        let index = slot as usize;
        let ticks = self.tick.saturating_sub(self.vehicles.spawn_tick[index]);
        let travel = ticks as f64 * DT;
        self.stats
            .record_arrival(travel, self.vehicles.free_flow[index]);
    }

    pub(super) fn accrue_delay(&mut self) {
        let mut delay = 0.0;
        for slot in self.vehicles.live_slots() {
            let desired =
                self.reference_speed(slot) * self.vehicles.kind[slot as usize].speed_factor();
            delay += DT * (1.0 - self.vehicles.v[slot as usize] / desired).max(0.0);
        }
        self.stats.accrued_delay += delay;
    }

    pub(super) fn extend_reference_speeds(&mut self) {
        self.v_ref.truncate(self.network.roads.count());
        let known = self.v_ref.len();
        let speeds = self.network.roads.speed.get(known..).unwrap_or_default();
        self.v_ref.extend_from_slice(speeds);
    }

    fn reference_speed(&self, slot: u32) -> f64 {
        let link = match self.vehicles.place[slot as usize] {
            Place::Link { link, .. } => Some(link),
            Place::Movement { .. } => self.vehicles.route_link(slot, 1),
        };
        link.and_then(|link| self.v_ref.get(road_of(link) as usize))
            .map_or(MIN_LINK_SPEED, |&speed| speed.max(MIN_LINK_SPEED))
    }

    fn live_speeds(&self) -> LiveSpeeds {
        let mut live = LiveSpeeds::default();
        for slot in self.vehicles.live_slots() {
            let v = self.vehicles.v[slot as usize];
            live.active += 1;
            live.speed_sum += v;
            live.stopped += u32::from(v < STOPPED_SPEED);
        }
        live
    }
}
