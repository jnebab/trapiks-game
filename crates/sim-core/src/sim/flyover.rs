use crate::edit::vehicles::record_movements;
use crate::edit::{Edit, EditOutcome, FlyoverUndo, restore_through, truncate_flyover};
use crate::network::{LinkId, direction_of, link_id, reverse, road_of};
use crate::vehicle::Place;

use super::Sim;
use super::edits::outcome;

#[derive(Clone, Copy)]
struct Splice {
    link: LinkId,
    s: f64,
    cursor: usize,
}

impl Sim {
    pub(super) fn apply_flyover(
        &mut self,
        edit: &Edit,
        (node, through): (u32, [u32; 2]),
        cost: i64,
    ) -> (Edit, EditOutcome) {
        let scope = edit.scope(&self.network);
        let held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        let arriving = through.map(|road| reverse(self.network.departing_link(road, node)));
        let inverse = edit.apply(&mut self.network);
        self.network.commit_edit(&scope.invalidated, scope.signals);
        for (&link, &inner) in arriving.iter().zip(&scope.roads[2..]) {
            self.move_onto_inner(link, inner);
        }
        self.settle_vehicles(&scope.roads, &scope.invalidated, &held);
        (inverse, outcome(scope, cost))
    }

    fn move_onto_inner(&mut self, arriving: LinkId, inner: u32) {
        let outer_len = self.network.link_length(arriving);
        let inner_len = self.network.roads.length[inner as usize];
        let inner_arriving = link_id(inner, direction_of(arriving));
        let departing = reverse(arriving);
        let slots: Vec<u32> = self.vehicles.live_slots().collect();
        for slot in slots {
            let index = slot as usize;
            let Place::Link { link, .. } = self.vehicles.place[index] else {
                continue;
            };
            let s = self.vehicles.s[index];
            let cursor = self.vehicles.cursor(slot);
            let splice = if link == arriving && s >= outer_len {
                Some(Splice {
                    link: inner_arriving,
                    s: s - outer_len,
                    cursor: cursor + 1,
                })
            } else if link == departing && s < inner_len {
                let inner_departing = reverse(inner_arriving);
                Some(Splice {
                    link: inner_departing,
                    s,
                    cursor,
                })
            } else if link == departing {
                self.vehicles.s[index] = s - inner_len;
                None
            } else {
                None
            };
            if let Some(splice) = splice {
                self.splice_vehicle(slot, splice);
            }
        }
    }

    fn splice_vehicle(&mut self, slot: u32, splice: Splice) {
        let mut route = self.vehicles.route(slot).to_vec();
        let at = splice.cursor.min(route.len());
        route.insert(at, splice.link);
        let splice = Splice {
            cursor: at,
            ..splice
        };
        self.relocate(slot, splice, &route);
    }

    fn relocate(&mut self, slot: u32, splice: Splice, route: &[LinkId]) {
        let Splice { link, s, cursor } = splice;
        let index = slot as usize;
        let Place::Link { lane, .. } = self.vehicles.place[index] else {
            return;
        };
        if !self.vehicles.rewrite_route(slot, route, cursor) {
            return self.strand(slot);
        }
        self.vehicles.set_place(slot, Place::Link { link, lane });
        self.vehicles.s[index] = s;
    }

    pub(super) fn undo_flyover(&mut self, undo: &FlyoverUndo, cost: i64) -> (Edit, EditOutcome) {
        let scope = Edit::UndoFlyover(Box::new(undo.clone())).scope(&self.network);
        let mut held = record_movements(&self.network, &self.vehicles, &scope.invalidated);
        self.move_off_inner(undo);
        restore_through(&mut self.network, undo);
        self.network.commit_edit(&scope.invalidated, true);
        self.ensure_graph();
        self.strand_on_flyover(undo);
        self.reroute_off_flyover(undo);
        truncate_flyover(&mut self.network, undo);
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
        self.settle_vehicles(&undo.roads, &surviving, &held);
        let inverse = Edit::BuildFlyover {
            node: undo.node,
            through: undo.roads,
        };
        (inverse, outcome(scope, cost))
    }

    fn move_off_inner(&mut self, undo: &FlyoverUndo) {
        let slots: Vec<u32> = self.vehicles.live_slots().collect();
        for slot in slots {
            let Place::Link { link, .. } = self.vehicles.place[slot as usize] else {
                continue;
            };
            if let Some((target, shift)) = self.restored_place(undo, link) {
                let (route, cursor) =
                    restored_route(undo, self.vehicles.route(slot), self.vehicles.cursor(slot));
                let s = self.vehicles.s[slot as usize] + shift;
                let splice = Splice {
                    link: target,
                    s,
                    cursor,
                };
                self.relocate(slot, splice, &route);
            }
        }
    }

    fn restored_place(&self, undo: &FlyoverUndo, link: LinkId) -> Option<(LinkId, f64)> {
        for index in 0..2 {
            let road = undo.roads[index];
            let middle = undo.old_ends[index];
            if road_of(link) == undo.inner(index) {
                let toward_top = self.network.link_to(link) == undo.top();
                let shift = if toward_top {
                    self.network.roads.length[road as usize]
                } else {
                    0.0
                };
                return Some((restore_link(undo, link), shift));
            }
            if road_of(link) == road && self.network.link_from(link) == middle {
                return Some((link, self.network.roads.length[undo.inner(index) as usize]));
            }
        }
        None
    }

    fn strand_on_flyover(&mut self, undo: &FlyoverUndo) {
        let appended = undo.appended_nodes();
        let slots: Vec<u32> = self
            .vehicles
            .live_slots()
            .filter(|&slot| match self.vehicles.place[slot as usize] {
                Place::Movement { node, .. } => appended.contains(&node),
                Place::Link { .. } => false,
            })
            .collect();
        for slot in slots {
            self.strand(slot);
        }
    }

    fn reroute_off_flyover(&mut self, undo: &FlyoverUndo) {
        let slots: Vec<u32> = self
            .vehicles
            .live_slots()
            .filter(|&slot| {
                let route = self.vehicles.route(slot);
                let rest = route.get(self.vehicles.cursor(slot)..).unwrap_or_default();
                rest.iter().any(|&link| undo.is_appended_link(link))
            })
            .collect();
        for slot in slots {
            self.reroute(slot);
        }
    }

    fn forget_truncated_links(&mut self) {
        let limit = self.network.link_count() as LinkId;
        let dropped = self.demand.trips.forget_links_from(limit)
            + self.demand.queues.forget_links_from(limit);
        self.stats.unserved += dropped;
    }
}

fn restore_link(undo: &FlyoverUndo, link: LinkId) -> LinkId {
    let index = (road_of(link) - undo.road_len_before) as usize;
    link_id(undo.roads[index], direction_of(link))
}

fn restored_route(undo: &FlyoverUndo, route: &[LinkId], cursor: usize) -> (Vec<LinkId>, usize) {
    let mut restored: Vec<LinkId> = Vec::with_capacity(route.len());
    let mut new_cursor = 0;
    for (position, &link) in route.iter().enumerate() {
        let mapped = if undo.is_appended_link(link) {
            restore_link(undo, link)
        } else {
            link
        };
        if restored.last() != Some(&mapped) {
            restored.push(mapped);
        }
        if position == cursor {
            new_cursor = restored.len() - 1;
        }
    }
    (restored, new_cursor)
}
