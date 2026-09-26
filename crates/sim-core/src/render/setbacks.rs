use crate::network::Network;

use super::JUNCTION_MIN_DEGREE;

pub fn road_setbacks(network: &Network) -> Vec<f32> {
    (0..network.roads.count() as u32)
        .flat_map(|road| road_ends(network, road))
        .collect()
}

fn road_ends(network: &Network, road: u32) -> [f32; 2] {
    if !network.is_road_active(road) {
        return [0.0, 0.0];
    }
    let index = road as usize;
    [network.roads.from[index], network.roads.to[index]]
        .map(|node| end_setback(network, node, road))
}

fn end_setback(network: &Network, node: u32, road: u32) -> f32 {
    if network.active_degree(node) < JUNCTION_MIN_DEGREE {
        return 0.0;
    }
    network.setback_at(node, road) as f32
}
