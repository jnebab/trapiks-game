use crate::consts::LANE_WIDTH;
use crate::map::Control;
use crate::network::{LinkId, Network, road_of};

use super::JUNCTION_MIN_DEGREE;

pub const MARKER_NONE: u8 = 0;
pub const MARKER_YIELD: u8 = 1;
pub const MARKER_STOP: u8 = 2;
pub const MARKER_SIGNAL: u8 = 3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApproachMarkers {
    pub link: Vec<u32>,
    pub node: Vec<u32>,
    pub kind: Vec<u8>,
    pub x1: Vec<f32>,
    pub y1: Vec<f32>,
    pub x2: Vec<f32>,
    pub y2: Vec<f32>,
}

pub fn approach_markers(network: &Network) -> ApproachMarkers {
    let mut markers = ApproachMarkers::default();
    for link in 0..network.link_count() as LinkId {
        add_marker(network, link, &mut markers);
    }
    markers
}

pub fn markers_at(network: &Network, nodes: &[u32]) -> ApproachMarkers {
    let mut markers = ApproachMarkers::default();
    for &node in nodes {
        for link in network.incoming(node) {
            add_marker(network, link, &mut markers);
        }
    }
    markers
}

fn add_marker(network: &Network, link: LinkId, markers: &mut ApproachMarkers) {
    if !network.is_link_active(link) {
        return;
    }
    let node = network.link_to(link);
    if network.active_degree(node) < JUNCTION_MIN_DEGREE {
        return;
    }
    let kind = marker_kind(network, link, node);
    if kind == MARKER_NONE {
        return;
    }
    let road = road_of(link);
    let (centre, dir) = network.centre_pose(link, network.link_span(link).1);
    let outer = network.roads.width(road) / 2.0;
    let inner = outer - f64::from(network.link_lanes(link)) * LANE_WIDTH;
    let normal = dir.perp_right();
    let (a, b) = (centre + normal * outer, centre + normal * inner);
    markers.link.push(link);
    markers.node.push(node);
    markers.kind.push(kind);
    markers.x1.push(a.x as f32);
    markers.y1.push(a.y as f32);
    markers.x2.push(b.x as f32);
    markers.y2.push(b.y as f32);
}

fn marker_kind(network: &Network, link: LinkId, node: u32) -> u8 {
    if network.signals().is_internal(link) {
        return MARKER_NONE;
    }
    if network.signal_state(link, 0).is_some() {
        return MARKER_SIGNAL;
    }
    let minor = is_minor(network, link, node);
    match network.nodes.control[node as usize] {
        Control::Priority | Control::Yield if minor => MARKER_YIELD,
        Control::Stop if minor => MARKER_STOP,
        Control::AllWayStop => MARKER_STOP,
        _ => MARKER_NONE,
    }
}

fn is_minor(network: &Network, link: LinkId, node: u32) -> bool {
    let rank = |l: LinkId| network.roads.class[road_of(l) as usize].rank();
    let highest = network.incoming(node).map(rank).max();
    highest.is_some_and(|top| rank(link) < top)
}
