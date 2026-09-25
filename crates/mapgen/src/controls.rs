use std::collections::BTreeMap;

use trapiks_sim_core::map::Control;

use crate::input::OsmData;
use crate::osm::Tags;
use crate::tags::tag;
use crate::topology::TopoRoad;

const CONTROL_RADIUS_M: f64 = 25.0;

pub type Positions = BTreeMap<i64, (f64, f64)>;

pub fn assign(osm: &OsmData, roads: &[TopoRoad], positions: &Positions) -> BTreeMap<i64, Control> {
    let mut controls = BTreeMap::new();
    for road in roads {
        assign_road(osm, road, positions, &mut controls);
    }
    controls
}

pub fn node_control(tags: &Tags) -> Option<Control> {
    match tag(tags, "highway")? {
        "traffic_signals" => Some(Control::Signal),
        "stop" if tag(tags, "stop") == Some("all") => Some(Control::AllWayStop),
        "stop" => Some(Control::Stop),
        "give_way" => Some(Control::Yield),
        _ => None,
    }
}

fn assign_road(
    osm: &OsmData,
    road: &TopoRoad,
    positions: &Positions,
    controls: &mut BTreeMap<i64, Control>,
) {
    let distances = arc_lengths(&road.node_ids, positions);
    let total = distances[distances.len() - 1];
    let last = road.node_ids.len() - 1;
    for (index, node) in road.node_ids.iter().enumerate() {
        let Some(control) = osm.nodes.get(node).and_then(|n| node_control(&n.tags)) else {
            continue;
        };
        if index == 0 || index == last {
            apply(controls, *node, control);
            continue;
        }
        if distances[index] <= CONTROL_RADIUS_M {
            apply(controls, road.from_node(), control);
        }
        if total - distances[index] <= CONTROL_RADIUS_M {
            apply(controls, road.to_node(), control);
        }
    }
}

fn apply(controls: &mut BTreeMap<i64, Control>, node: i64, control: Control) {
    let entry = controls.entry(node).or_insert(control);
    *entry = (*entry).max(control);
}

fn arc_lengths(nodes: &[i64], positions: &Positions) -> Vec<f64> {
    let mut distances = Vec::with_capacity(nodes.len());
    let mut total = 0.0;
    let mut previous: Option<(f64, f64)> = None;
    for node in nodes {
        let point = positions.get(node).copied().unwrap_or_default();
        if let Some((px, py)) = previous {
            total += (point.0 - px).hypot(point.1 - py);
        }
        distances.push(total);
        previous = Some(point);
    }
    distances
}
