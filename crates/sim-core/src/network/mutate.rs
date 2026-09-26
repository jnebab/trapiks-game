use crate::map::{Control, TurnBan};

use super::{Network, SpatialGrid, Timing};

impl Network {
    pub(crate) fn set_road_deleted(&mut self, road: u32, deleted: bool) -> bool {
        std::mem::replace(&mut self.roads.deleted[road as usize], deleted)
    }

    pub(crate) fn set_road_lanes(&mut self, road: u32, forward: u8, backward: u8) -> (u8, u8) {
        let index = road as usize;
        let old_forward = std::mem::replace(&mut self.roads.lanes_forward[index], forward);
        let old_backward = std::mem::replace(&mut self.roads.lanes_backward[index], backward);
        (old_forward, old_backward)
    }

    pub(crate) fn set_road_speed(&mut self, road: u32, kph: u8) -> u8 {
        let old = self.roads.speed_kph[road as usize];
        self.roads.set_speed_kph(road, kph);
        old
    }

    pub(crate) fn set_control(&mut self, node: u32, control: Control) -> Control {
        std::mem::replace(&mut self.nodes.control[node as usize], control)
    }

    pub(crate) fn set_timing(&mut self, key: u32, timing: Option<Timing>) -> Option<Timing> {
        match timing {
            Some(timing) => self.timings.insert(key, timing),
            None => self.timings.remove(&key),
        }
    }

    pub(crate) fn set_ban(&mut self, ban: TurnBan, banned: bool) -> bool {
        let was = self.bans.contains(&ban);
        if banned {
            self.bans.insert(ban);
        } else {
            self.bans.remove(&ban);
        }
        was
    }

    pub(crate) fn commit_edit(&mut self, invalidated: &[u32], signals_changed: bool) {
        let structural = self.structure != self.spatial_structure;
        if structural {
            self.spatial = SpatialGrid::build(&self.roads, &self.nodes);
            self.spatial_structure = self.structure;
        }
        for &node in invalidated {
            self.invalidate_node(node);
        }
        if structural {
            self.compute_spans();
        }
        if signals_changed {
            self.rebuild_signals();
        }
        self.version += 1;
    }

    pub fn is_signal_node(&self, node: u32) -> bool {
        self.nodes.control.get(node as usize) == Some(&Control::Signal)
    }
}
