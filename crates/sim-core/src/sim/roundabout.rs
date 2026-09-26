use crate::edit::vehicles::record_movements;
use crate::edit::{Edit, EditOutcome, RoundaboutUndo, restore_arms, truncate_roundabout};
use crate::network::{LinkId, reverse};
use crate::vehicle::Place;

use super::Sim;
use super::edits::outcome;

#[derive(Clone, Copy)]
struct ArmLinks {
    toward: LinkId,
    away: LinkId,
    cut: f64,
}

impl Sim {
    pub(super) fn apply_roundabout(
        &mut self,
        edit: &Edit,
        (node, radius_m): (u32, u8),
        cost: i64,
    ) -> (Edit, EditOutcome) {
        let scope = edit.scope(&self.network);
        let held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        let arms: Vec<ArmLinks> = self.network.nodes.roads[node as usize]
            .iter()
            .filter(|&&road| scope.roads.contains(&road))
            .map(|&road| {
                let away = self.network.departing_link(road, node);
                ArmLinks {
                    toward: reverse(away),
                    away,
                    cut: f64::from(radius_m),
                }
            })
            .collect();
        let inverse = edit.apply(&mut self.network);
        self.network.commit_edit(&scope.invalidated, scope.signals);
        for arm in arms {
            self.shorten_arm(arm);
        }
        self.settle_vehicles(&scope.roads, &scope.invalidated, &held);
        (inverse, outcome(scope, cost))
    }

    fn shorten_arm(&mut self, arm: ArmLinks) {
        let length = self.network.link_length(arm.toward);
        let slots: Vec<u32> = self.vehicles.live_slots().collect();
        for slot in slots {
            let index = slot as usize;
            let Place::Link { link, .. } = self.vehicles.place[index] else {
                continue;
            };
            let s = self.vehicles.s[index];
            if (link == arm.toward && s > length) || (link == arm.away && s < arm.cut) {
                self.strand(slot);
            } else if link == arm.away {
                self.vehicles.s[index] = s - arm.cut;
            }
        }
    }

    pub(super) fn undo_roundabout(
        &mut self,
        undo: &RoundaboutUndo,
        cost: i64,
    ) -> (Edit, EditOutcome) {
        let scope = Edit::UndoRoundabout(Box::new(undo.clone())).scope(&self.network);
        let mut held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        self.strand_on_ring(undo);
        self.lengthen_arms(undo);
        restore_arms(&mut self.network, undo);
        self.network.commit_edit(&scope.invalidated, true);
        self.ensure_graph();
        self.reroute_off_links_from(undo.road_len_before * 2);
        truncate_roundabout(&mut self.network, undo);
        let surviving: Vec<u32> = scope
            .invalidated
            .iter()
            .copied()
            .filter(|&node| node < undo.node_len_before)
            .collect();
        self.network.commit_edit(&surviving, true);
        self.forget_truncated_links();
        held.retain(|entry| {
            self.vehicles.alive[entry.slot as usize] && entry.node < undo.node_len_before
        });
        let arms: Vec<u32> = undo.arms.iter().map(|arm| arm.road).collect();
        self.settle_vehicles(&arms, &surviving, &held);
        (undo.rebuild(), outcome(scope, cost))
    }

    fn strand_on_ring(&mut self, undo: &RoundaboutUndo) {
        self.strand_in_movements_at(&undo.ring_nodes());
        let slots: Vec<u32> = self
            .vehicles
            .live_slots()
            .filter(|&slot| match self.vehicles.place[slot as usize] {
                Place::Link { link, .. } => undo.is_ring_link(link),
                Place::Movement { .. } => false,
            })
            .collect();
        for slot in slots {
            self.strand(slot);
        }
    }

    fn lengthen_arms(&mut self, undo: &RoundaboutUndo) {
        for (index, arm) in undo.arms.iter().enumerate() {
            let away = self.network.departing_link(arm.road, undo.ring_node(index));
            self.shift_on(away, arm.cut);
        }
    }

    fn shift_on(&mut self, target: LinkId, shift: f64) {
        let slots: Vec<u32> = self.vehicles.live_slots().collect();
        for slot in slots {
            let index = slot as usize;
            if matches!(self.vehicles.place[index], Place::Link { link, .. } if link == target) {
                self.vehicles.s[index] += shift;
            }
        }
    }
}
