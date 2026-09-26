use crate::geom::Vec2;
use crate::map::Control;

use super::Network;
use super::node::NodeStore;
use super::road::RoadStore;

impl RoadStore {
    pub fn append_like(
        &mut self,
        r: u32,
        (from, to): (u32, u32),
        points: &[Vec2],
        layer: i8,
    ) -> u32 {
        let index = r as usize;
        let id = self.count() as u32;
        self.from.push(from);
        self.to.push(to);
        self.class.push(self.class[index]);
        self.lanes_forward.push(self.lanes_forward[index]);
        self.lanes_backward.push(self.lanes_backward[index]);
        self.layer.push(layer);
        self.name.push(self.name[index]);
        self.speed_kph.push(self.speed_kph[index]);
        self.speed.push(self.speed[index]);
        self.deleted.push(false);
        let (range, length) = self.push_points(points);
        self.point_range.push(range);
        self.length.push(length);
        id
    }

    fn truncate(&mut self, len: usize) {
        self.from.truncate(len);
        self.to.truncate(len);
        self.class.truncate(len);
        self.lanes_forward.truncate(len);
        self.lanes_backward.truncate(len);
        self.layer.truncate(len);
        self.name.truncate(len);
        self.speed_kph.truncate(len);
        self.speed.truncate(len);
        self.deleted.truncate(len);
        self.point_range.truncate(len);
        self.length.truncate(len);
    }
}

impl NodeStore {
    pub fn append(&mut self, pos: Vec2) -> u32 {
        let id = self.count() as u32;
        self.pos.push(pos);
        self.control.push(Control::Priority);
        self.roads.push(Vec::new());
        id
    }

    fn truncate(&mut self, len: usize) {
        self.pos.truncate(len);
        self.control.truncate(len);
        self.roads.truncate(len);
    }

    fn attach_sorted(&mut self, node: u32, road: u32) {
        let Some(list) = self.roads.get_mut(node as usize) else {
            return;
        };
        if let Err(at) = list.binary_search(&road) {
            list.insert(at, road);
        }
    }

    fn detach(&mut self, node: u32, road: u32) {
        let Some(list) = self.roads.get_mut(node as usize) else {
            return;
        };
        if let Ok(at) = list.binary_search(&road) {
            list.remove(at);
        }
    }
}

impl Network {
    pub(crate) fn append_node(&mut self, pos: Vec2) -> u32 {
        self.junctions.push(None);
        self.structure += 1;
        self.nodes.append(pos)
    }

    pub(crate) fn append_road(
        &mut self,
        r: u32,
        ends: (u32, u32),
        points: &[Vec2],
        layer: i8,
    ) -> u32 {
        let road = self.roads.append_like(r, ends, points, layer);
        self.nodes.attach_sorted(ends.0, road);
        self.nodes.attach_sorted(ends.1, road);
        if let Some(mask) = self.region.as_mut() {
            mask.active_road.push(true);
        }
        self.structure += 1;
        road
    }

    pub(crate) fn set_endpoint(&mut self, road: u32, old_node: u32, new_node: u32) {
        let index = road as usize;
        if self.roads.from[index] == old_node {
            self.roads.from[index] = new_node;
        }
        if self.roads.to[index] == old_node {
            self.roads.to[index] = new_node;
        }
        self.nodes.detach(old_node, road);
        self.nodes.attach_sorted(new_node, road);
        self.structure += 1;
    }

    pub(crate) fn repoint_road(&mut self, road: u32, points: &[Vec2]) -> ((u32, u32), f64) {
        let (range, length) = self.roads.push_points(points);
        self.restore_geometry(road, range, length)
    }

    pub(crate) fn restore_geometry(
        &mut self,
        road: u32,
        range: (u32, u32),
        length: f64,
    ) -> ((u32, u32), f64) {
        let index = road as usize;
        let old = (self.roads.point_range[index], self.roads.length[index]);
        self.roads.set_geometry(road, range, length);
        self.structure += 1;
        old
    }

    pub(crate) fn truncate_roads(&mut self, len: usize) {
        for road in len..self.roads.count() {
            let (from, to) = (self.roads.from[road], self.roads.to[road]);
            self.nodes.detach(from, road as u32);
            self.nodes.detach(to, road as u32);
        }
        self.roads.truncate(len);
        if let Some(mask) = self.region.as_mut() {
            mask.active_road.truncate(len);
        }
        self.structure += 1;
    }

    pub(crate) fn truncate_nodes(&mut self, len: usize) {
        self.nodes.truncate(len);
        self.junctions.truncate(len);
        self.structure += 1;
    }
}
