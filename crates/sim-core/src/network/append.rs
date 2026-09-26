use crate::geom::Vec2;
use crate::map::{Control, RoadClass};

use super::Network;
use super::node::NodeStore;
use super::road::RoadStore;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NewRoad {
    pub ends: (u32, u32),
    pub class: RoadClass,
    pub lanes: (u8, u8),
    pub layer: i8,
    pub name: u32,
    pub speed_kph: u8,
    pub roundabout: bool,
}

impl RoadStore {
    fn like(&self, r: u32, ends: (u32, u32), layer: i8) -> NewRoad {
        let index = r as usize;
        NewRoad {
            ends,
            class: self.class[index],
            lanes: (self.lanes_forward[index], self.lanes_backward[index]),
            layer,
            name: self.name[index],
            speed_kph: self.speed_kph[index],
            roundabout: self.roundabout[index],
        }
    }

    pub fn append_like(&mut self, r: u32, ends: (u32, u32), points: &[Vec2], layer: i8) -> u32 {
        let row = self.like(r, ends, layer);
        self.append(&row, points)
    }

    pub fn append(&mut self, row: &NewRoad, points: &[Vec2]) -> u32 {
        let id = self.count() as u32;
        self.from.push(row.ends.0);
        self.to.push(row.ends.1);
        self.class.push(row.class);
        self.lanes_forward.push(row.lanes.0);
        self.lanes_backward.push(row.lanes.1);
        self.layer.push(row.layer);
        self.name.push(row.name);
        self.speed_kph.push(row.speed_kph);
        self.speed.push(0.0);
        self.roundabout.push(row.roundabout);
        self.deleted.push(false);
        self.set_speed_kph(id, row.speed_kph);
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
        self.roundabout.truncate(len);
        self.deleted.truncate(len);
        self.point_range.truncate(len);
        self.length.truncate(len);
    }
}

impl NodeStore {
    pub fn append(&mut self, pos: Vec2, control: Control) -> u32 {
        let id = self.count() as u32;
        self.pos.push(pos);
        self.control.push(control);
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
    pub(crate) fn append_node(&mut self, pos: Vec2, control: Control) -> u32 {
        self.junctions.push(None);
        self.structure += 1;
        self.nodes.append(pos, control)
    }

    pub(crate) fn append_road(
        &mut self,
        r: u32,
        ends: (u32, u32),
        points: &[Vec2],
        layer: i8,
    ) -> u32 {
        let road = self.roads.append_like(r, ends, points, layer);
        self.attach_appended(road, ends)
    }

    pub(crate) fn append_new_road(&mut self, row: &NewRoad, points: &[Vec2]) -> u32 {
        let road = self.roads.append(row, points);
        self.attach_appended(road, row.ends)
    }

    fn attach_appended(&mut self, road: u32, ends: (u32, u32)) -> u32 {
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
