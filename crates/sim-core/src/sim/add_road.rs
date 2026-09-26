use crate::edit::vehicles::{HeldMovement, record_movements};
use crate::edit::{AddRoadUndo, Edit, EditOutcome};
use crate::network::LinkId;
use crate::vehicle::Place;

use super::Sim;
use super::edits::outcome;
use super::split_routes::{SpliceRole, SplitLinks, splice_route};

impl Sim {
    pub(super) fn apply_add_road(&mut self, edit: &Edit, cost: i64) -> (Edit, EditOutcome) {
        let scope = edit.scope(&self.network);
        let mut held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        let inverse = edit.apply(&mut self.network);
        self.network.commit_edit(&scope.invalidated, scope.signals);
        let links = split_links(&inverse);
        for split in &links {
            substitute(&mut held, split.old_to, |link| split.split_of(link));
        }
        for split in &links {
            self.split_vehicles(split);
        }
        self.settle_vehicles(&scope.roads, &scope.invalidated, &held);
        (inverse, outcome(scope, cost))
    }

    fn split_vehicles(&mut self, links: &SplitLinks) {
        let slots: Vec<u32> = self.vehicles.live_slots().collect();
        for slot in slots {
            self.split_vehicle(slot, links);
        }
    }

    fn split_vehicle(&mut self, slot: u32, links: &SplitLinks) {
        let index = slot as usize;
        let s = self.vehicles.s[index];
        let cursor = self.vehicles.cursor(slot);
        let current = self.vehicles.route(slot).get(cursor).copied();
        let (target, role) = match self.vehicles.place[index] {
            Place::Link { link, .. } if link == links.fwd && s >= links.at => {
                (Some((links.fwd2, s - links.at)), SpliceRole::FollowsForward)
            }
            Place::Link { link, .. } if link == links.bwd && s < links.len - links.at => {
                (Some((links.bwd2, s)), SpliceRole::OnSecondBackward)
            }
            Place::Link { link, .. } if link == links.bwd => {
                self.vehicles.s[index] = s - (links.len - links.at);
                (None, SpliceRole::Other)
            }
            Place::Movement { node, .. } if node == links.old_to && current == Some(links.fwd) => {
                (None, SpliceRole::FollowsForward)
            }
            _ => (None, SpliceRole::Other),
        };
        let route = self.vehicles.route(slot);
        let (spliced, new_cursor) = splice_route(route, cursor, links, role);
        if spliced.len() == route.len() {
            return;
        }
        if !self.vehicles.splice_route(slot, &spliced, new_cursor) {
            return self.strand(slot);
        }
        if let Some((link, s)) = target {
            self.move_vehicle(slot, link, s);
        }
    }

    pub(super) fn move_vehicle(&mut self, slot: u32, target: LinkId, s: f64) {
        let index = slot as usize;
        let Place::Link { lane, .. } = self.vehicles.place[index] else {
            return;
        };
        self.vehicles
            .set_place(slot, Place::Link { link: target, lane });
        self.vehicles.s[index] = s;
        self.vehicles.reset_junction_state(slot);
    }
}

pub(super) fn split_links(inverse: &Edit) -> Vec<SplitLinks> {
    match inverse {
        Edit::UndoAddRoad(undo) => undo_links(undo),
        _ => Vec::new(),
    }
}

pub(super) fn undo_links(undo: &AddRoadUndo) -> Vec<SplitLinks> {
    undo.splits.iter().map(SplitLinks::new).collect()
}

pub(super) fn substitute(held: &mut [HeldMovement], node: u32, map: impl Fn(LinkId) -> LinkId) {
    for entry in held.iter_mut().filter(|entry| entry.node == node) {
        entry.from = map(entry.from);
        entry.to = map(entry.to);
    }
}
