use crate::geom::Vec2;

use super::road::RoadStore;

pub type LinkId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward = 0,
    Backward = 1,
}

pub fn link_id(road: u32, dir: Direction) -> LinkId {
    road * 2 + dir as u32
}

pub fn road_of(link: LinkId) -> u32 {
    link / 2
}

pub fn direction_of(link: LinkId) -> Direction {
    if link & 1 == 0 {
        Direction::Forward
    } else {
        Direction::Backward
    }
}

pub fn reverse(link: LinkId) -> LinkId {
    link ^ 1
}

pub fn lanes(roads: &RoadStore, link: LinkId) -> u8 {
    let road = road_of(link) as usize;
    match direction_of(link) {
        Direction::Forward => roads.lanes_forward[road],
        Direction::Backward => roads.lanes_backward[road],
    }
}

pub fn from_node(roads: &RoadStore, link: LinkId) -> u32 {
    let road = road_of(link) as usize;
    match direction_of(link) {
        Direction::Forward => roads.from[road],
        Direction::Backward => roads.to[road],
    }
}

pub fn to_node(roads: &RoadStore, link: LinkId) -> u32 {
    from_node(roads, reverse(link))
}

pub fn arriving(roads: &RoadStore, road: u32, node: u32) -> LinkId {
    let forward = link_id(road, Direction::Forward);
    if to_node(roads, forward) == node {
        return forward;
    }
    reverse(forward)
}

pub fn departing(roads: &RoadStore, road: u32, node: u32) -> LinkId {
    reverse(arriving(roads, road, node))
}

pub fn length(roads: &RoadStore, link: LinkId) -> f64 {
    roads.length[road_of(link) as usize]
}

pub fn centre_pose(roads: &RoadStore, link: LinkId, s: f64) -> (Vec2, Vec2) {
    let road = road_of(link);
    let points = roads.points(road);
    let cumulative = roads.cumulative(road);
    match direction_of(link) {
        Direction::Forward => crate::geom::pose_at(points, cumulative, s),
        Direction::Backward => {
            let (pos, tangent) = crate::geom::pose_at(points, cumulative, length(roads, link) - s);
            (pos, -tangent)
        }
    }
}
