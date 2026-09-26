use crate::consts::{IDM_MIN_GAP, STOP_LINE_WINDOW, STOPPED_SPEED};
use crate::network::LinkId;
use crate::vehicle::approach::{PriorityKey, approach, priority_key, within_zone};
use crate::vehicle::entry::{Verdict, evaluate};
use crate::vehicle::{Place, RuleContext};

use super::Sim;

impl Sim {
    pub fn rule_context(&self) -> RuleContext<'_> {
        RuleContext {
            network: &self.network,
            vehicles: &self.vehicles,
            occupancy: &self.occupancy,
            tick: self.tick,
        }
    }

    pub(super) fn update_zones(&mut self) {
        let mut zone = std::mem::take(&mut self.zone);
        zone.clear();
        zone.resize(self.vehicles.slot_count(), false);
        for slot in 0..self.vehicles.slot_count() as u32 {
            if self.vehicles.alive[slot as usize] {
                zone[slot as usize] = self.update_zone(slot);
            }
        }
        self.zone = zone;
    }

    fn update_zone(&mut self, slot: u32) -> bool {
        let index = slot as usize;
        let Place::Link { link, .. } = self.vehicles.place[index] else {
            return false;
        };
        let (end, next) = self.link_end_and_next(slot, link);
        let v = self.vehicles.v[index];
        let remaining = end - self.vehicles.s[index];
        let zone = next.is_some() && within_zone(remaining, v);
        if zone && self.vehicles.arrival_tick[index] == u64::MAX {
            self.vehicles.arrival_tick[index] = self.tick;
        }
        if v < STOPPED_SPEED && remaining <= STOP_LINE_WINDOW + IDM_MIN_GAP {
            self.vehicles.stopped_at_line[index] = true;
        }
        zone
    }

    fn in_zone(&self, slot: u32) -> bool {
        self.zone.get(slot as usize).copied().unwrap_or(false)
    }

    fn link_end_and_next(&self, slot: u32, link: LinkId) -> (f64, Option<LinkId>) {
        let ahead = self.ahead_of(slot);
        let end = ahead
            .end()
            .unwrap_or_else(|| self.network.link_span(link).1);
        (end, ahead.next())
    }

    pub(super) fn decide(&mut self) {
        let mut candidates = std::mem::take(&mut self.candidates);
        self.collect_candidates(&mut candidates);
        candidates.sort_unstable_by_key(|&(key, _)| key);
        for &(_, slot) in &candidates {
            self.decide_one(slot);
        }
        self.candidates = candidates;
    }

    fn collect_candidates(&self, out: &mut Vec<(PriorityKey, u32)>) {
        out.clear();
        let ctx = self.rule_context();
        for slot in self.vehicles.live_slots() {
            if self.vehicles.committed[slot as usize] || !self.in_zone(slot) {
                continue;
            }
            if let Some(a) = approach(&ctx, slot) {
                out.push((priority_key(&ctx, &a), slot));
            }
        }
    }

    fn decide_one(&mut self, slot: u32) {
        let index = slot as usize;
        match evaluate(&self.rule_context(), slot) {
            Verdict::Pass => self.vehicles.committed[index] = true,
            Verdict::Yielding => self.vehicles.wait_ticks[index] += 1,
            Verdict::Blocked => {}
        }
    }

    pub(super) fn stop_line_gap(&self, slot: u32) -> Option<f64> {
        let index = slot as usize;
        if self.vehicles.committed[index] || !self.in_zone(slot) {
            return None;
        }
        let Place::Link { .. } = self.vehicles.place[index] else {
            return None;
        };
        Some(self.ahead_of(slot).end()? - self.vehicles.s[index])
    }
}
