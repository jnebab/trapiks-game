use crate::edit::vehicles::record_movements;
use crate::edit::{
    AddRoadUndo, Edit, EditOutcome, delete_added_road, restore_split, truncate_added_road,
};
use crate::network::{LinkId, road_of};
use crate::vehicle::Place;

use super::Sim;
use super::add_road::{substitute, undo_links};
use super::edits::outcome;
use super::split_routes::{SplitLinks, has_split_link, merge_route};

impl Sim {
    pub(super) fn undo_add_road(&mut self, undo: &AddRoadUndo, cost: i64) -> (Edit, EditOutcome) {
        let scope = Edit::UndoAddRoad(Box::new(undo.clone())).scope(&self.network);
        let mut held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        let links = undo_links(undo);
        for split in &links {
            substitute(&mut held, split.old_to, |link| split.whole_of(link));
        }
        self.strand_on_road(undo.new_road());
        for index in (0..links.len()).rev() {
            self.strand_in_movements_at(&[undo.split_node(index)]);
            self.merge_vehicles(&links[index]);
            restore_split(&mut self.network, undo, index);
        }
        delete_added_road(&mut self.network, undo);
        self.network.commit_edit(&scope.invalidated, true);
        self.ensure_graph();
        self.reroute_off_links_from(undo.road_len_before * 2);
        truncate_added_road(&mut self.network, undo);
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
        let roads: Vec<u32> = undo.splits.iter().map(|split| split.road).collect();
        self.settle_vehicles(&roads, &surviving, &held);
        (Edit::AddRoad(Box::new(undo.spec)), outcome(scope, cost))
    }

    fn strand_on_road(&mut self, road: u32) {
        let slots: Vec<u32> = self
            .vehicles
            .live_slots()
            .filter(|&slot| self.uses_road_now(slot, road))
            .collect();
        for slot in slots {
            self.strand(slot);
        }
    }

    fn uses_road_now(&self, slot: u32, road: u32) -> bool {
        let on_road = |link: Option<LinkId>| link.is_some_and(|link| road_of(link) == road);
        match self.vehicles.place[slot as usize] {
            Place::Link { link, .. } => road_of(link) == road,
            Place::Movement { .. } => {
                on_road(self.vehicles.route_link(slot, 0))
                    || on_road(self.vehicles.route_link(slot, 1))
            }
        }
    }

    fn merge_vehicles(&mut self, links: &SplitLinks) {
        let slots: Vec<u32> = self.vehicles.live_slots().collect();
        for slot in slots {
            self.merge_vehicle(slot, links);
        }
    }

    fn merge_vehicle(&mut self, slot: u32, links: &SplitLinks) {
        let index = slot as usize;
        let s = self.vehicles.s[index];
        let target = match self.vehicles.place[index] {
            Place::Link { link, .. } if link == links.fwd2 => Some((links.fwd, s + links.at)),
            Place::Link { link, .. } if link == links.bwd2 => Some((links.bwd, s)),
            Place::Link { link, .. } if link == links.bwd => {
                self.vehicles.s[index] = s + (links.len - links.at);
                None
            }
            _ => None,
        };
        let route = self.vehicles.route(slot);
        if !has_split_link(route, links) {
            return;
        }
        let (merged, cursor) = merge_route(route, self.vehicles.cursor(slot), links);
        if !self.vehicles.splice_route(slot, &merged, cursor) {
            return self.strand(slot);
        }
        if let Some((link, s)) = target {
            self.move_vehicle(slot, link, s);
        }
    }
}
