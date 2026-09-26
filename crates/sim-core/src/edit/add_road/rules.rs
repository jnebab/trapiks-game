use crate::geom::{Vec2, bounds, intersect, pose_at, signed_turn};
use crate::network::Network;

use super::super::apply::Edit;
use super::super::validate::{Planned, require};
use super::super::{EditError, Endpoint, Prepared};
use super::geometry::{AddRoadSpec, cost, plan_points, polyline_length, resolve, via_point};

pub const ADD_ROAD_MIN_ANGLE: f64 = 25.0;
const MAX_LANES_PER_SIDE: u8 = 4;
const MAX_LAYER: i8 = 2;
const END_MARGIN_M: f64 = 10.0;
const MAX_VIA_OFFSET_M: f64 = 1_000.0;
const MIN_LENGTH_M: f64 = 10.0;
const MAX_LENGTH_M: f64 = 2_000.0;
const CROSSING_SLACK_M: f64 = 2.0;

pub fn add_road(network: &Network, spec: &AddRoadSpec) -> Planned {
    for end in spec.ends() {
        check_bounds(network, end)?;
    }
    check_region(network, spec)?;
    check_shape(spec)?;
    for end in spec.ends() {
        check_endpoint(network, end)?;
    }
    require(!same_endpoint(spec), EditError::SameEndpoint)?;
    let points = checked_points(network, spec)?;
    let angled = spec
        .ends()
        .iter()
        .zip(end_directions(&points))
        .all(|(&end, direction)| clears_angle(network, end, direction));
    require(angled, EditError::AngleTooSharp)?;
    require(
        !crosses(network, &points, spec.layer),
        EditError::CrossesRoad,
    )?;
    Ok(Prepared::Edit {
        edit: Edit::AddRoad(Box::new(*spec)),
        cost: cost(spec, polyline_length(&points)),
    })
}

fn check_bounds(network: &Network, end: Endpoint) -> Result<(), EditError> {
    match end {
        Endpoint::Node { node } => require(
            (node as usize) < network.nodes.count(),
            EditError::NodeNotFound,
        ),
        Endpoint::OnRoad { road, .. } => {
            require(
                (road as usize) < network.roads.count(),
                EditError::RoadNotFound,
            )?;
            require(network.roads.is_live(road), EditError::RoadDeleted)
        }
    }
}

fn check_region(network: &Network, spec: &AddRoadSpec) -> Result<(), EditError> {
    let Some(mask) = network.region() else {
        return Ok(());
    };
    let inside = spec.ends().iter().all(|&end| {
        let outside = resolve(network, end).distance(mask.circle.center) > mask.circle.radius;
        let road_active = match end {
            Endpoint::Node { .. } => true,
            Endpoint::OnRoad { road, .. } => mask.active_road[road as usize],
        };
        road_active && !outside
    });
    require(inside, EditError::OutsideRegion)
}

fn check_shape(spec: &AddRoadSpec) -> Result<(), EditError> {
    let (forward, backward) = spec.lanes;
    let lanes_ok = u16::from(forward) + u16::from(backward) >= 1
        && forward <= MAX_LANES_PER_SIDE
        && backward <= MAX_LANES_PER_SIDE;
    require(lanes_ok, EditError::InvalidLanes)?;
    require(
        (0..=MAX_LAYER).contains(&spec.layer),
        EditError::InvalidLayer,
    )
}

fn check_endpoint(network: &Network, end: Endpoint) -> Result<(), EditError> {
    match end {
        Endpoint::Node { node } => require(
            network.active_degree(node) >= 1,
            EditError::EndpointIsolated,
        ),
        Endpoint::OnRoad { road, at_m } => {
            let length = network.roads.length[road as usize];
            let inside = at_m.is_finite() && at_m >= END_MARGIN_M && at_m <= length - END_MARGIN_M;
            require(inside, EditError::TooCloseToEnd)
        }
    }
}

fn same_endpoint(spec: &AddRoadSpec) -> bool {
    match (spec.from, spec.to) {
        (Endpoint::Node { node: a }, Endpoint::Node { node: b }) => a == b,
        (Endpoint::OnRoad { road: a, .. }, Endpoint::OnRoad { road: b, .. }) => a == b,
        _ => false,
    }
}

fn checked_points(network: &Network, spec: &AddRoadSpec) -> Result<Vec<Vec2>, EditError> {
    if let Some(via) = via_point(spec) {
        let chord_mid = Vec2::lerp(resolve(network, spec.from), resolve(network, spec.to), 0.5);
        let finite = via.x.is_finite() && via.y.is_finite();
        let near = via.distance(chord_mid) <= MAX_VIA_OFFSET_M;
        require(finite && near, EditError::InvalidLength)?;
    }
    let points = plan_points(network, spec);
    let length = polyline_length(&points);
    require(
        (MIN_LENGTH_M..=MAX_LENGTH_M).contains(&length),
        EditError::InvalidLength,
    )?;
    Ok(points)
}

fn end_directions(points: &[Vec2]) -> [Vec2; 2] {
    let n = points.len();
    [points[1] - points[0], points[n - 2] - points[n - 1]]
}

fn clears_angle(network: &Network, end: Endpoint, direction: Vec2) -> bool {
    let min = ADD_ROAD_MIN_ANGLE.to_radians();
    leaving_directions(network, end)
        .iter()
        .all(|&other| signed_turn(direction, other).abs() >= min)
}

fn leaving_directions(network: &Network, end: Endpoint) -> Vec<Vec2> {
    match end {
        Endpoint::Node { node } => network.nodes.roads[node as usize]
            .iter()
            .copied()
            .filter(|&road| network.is_road_active(road))
            .map(|road| departing_direction(network, road, node))
            .collect(),
        Endpoint::OnRoad { road, at_m } => {
            let (points, cumulative) = (network.roads.points(road), network.roads.cumulative(road));
            let tangent = pose_at(points, cumulative, at_m).1;
            vec![tangent, -tangent]
        }
    }
}

fn departing_direction(network: &Network, road: u32, node: u32) -> Vec2 {
    let points = network.roads.points(road);
    let n = points.len();
    if network.roads.from[road as usize] == node {
        points[1] - points[0]
    } else {
        points[n - 2] - points[n - 1]
    }
}

fn crosses(network: &Network, points: &[Vec2], layer: i8) -> bool {
    let (min, max) = bounds(points.iter().copied());
    let ends = [points[0], points[points.len() - 1]];
    network
        .spatial
        .roads_in_rect(min, max)
        .into_iter()
        .filter(|&road| network.roads.is_live(road) && network.roads.layer[road as usize] == layer)
        .any(|road| polylines_cross(points, network.roads.points(road), ends))
}

fn polylines_cross(new: &[Vec2], other: &[Vec2], ends: [Vec2; 2]) -> bool {
    new.windows(2).any(|a| {
        other.windows(2).any(|b| {
            intersect(a[0], a[1], b[0], b[1]).is_some_and(|(t, _)| {
                let hit = Vec2::lerp(a[0], a[1], t);
                ends.iter()
                    .all(|&end| hit.distance(end) >= CROSSING_SLACK_M)
            })
        })
    })
}
