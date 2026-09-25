use crate::geom::{Vec2, pose_at};
use crate::network::{Movement, Network, road_of};

use super::Place;

pub fn place_end(network: &Network, place: Place) -> Option<f64> {
    match place {
        Place::Link { link, .. } => Some(network.link_span(link).1),
        Place::Movement { node, movement, .. } => {
            Some(movement_of(network, node, movement)?.length)
        }
    }
}

pub fn movement_of(network: &Network, node: u32, movement: u16) -> Option<&Movement> {
    network.junction(node)?.movements.get(usize::from(movement))
}

pub fn pose(network: &Network, place: Place, s: f64) -> Option<(Vec2, f64)> {
    let (pos, tangent) = match place {
        Place::Link { link, lane } => network.link_pose(link, s, lane),
        Place::Movement {
            node,
            movement,
            from_lane,
            to_lane,
        } => {
            let movement = movement_of(network, node, movement)?;
            movement_pose(network, movement, s, (from_lane, to_lane))
        }
    };
    Some((pos, libm::atan2(tangent.y, tangent.x)))
}

fn movement_pose(network: &Network, movement: &Movement, s: f64, lanes: (u8, u8)) -> (Vec2, Vec2) {
    let (point, tangent) = pose_at(&movement.path, &movement.cumulative, s);
    let from_road = road_of(movement.from_link);
    let to_road = road_of(movement.to_link);
    let delta_from = network.lane_offset(from_road, lanes.0)
        - network.lane_offset(from_road, movement.primary_lanes.0);
    let delta_to = network.lane_offset(to_road, lanes.1)
        - network.lane_offset(to_road, movement.primary_lanes.1);
    let u = progress(s, movement.length);
    let smooth = u * u * (3.0 - 2.0 * u);
    let lateral = delta_from + (delta_to - delta_from) * smooth;
    (point + tangent.perp_right() * lateral, tangent)
}

fn progress(s: f64, length: f64) -> f64 {
    if length <= 0.0 {
        return 1.0;
    }
    (s / length).clamp(0.0, 1.0)
}
