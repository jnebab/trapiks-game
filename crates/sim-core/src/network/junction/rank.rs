use crate::map::Control;
use crate::network::Network;

use super::Movement;

pub const RING_RANK: u8 = 15;

pub fn movement_class_rank(network: &Network, road: u32) -> u8 {
    if network.roads.is_roundabout(road) {
        return RING_RANK;
    }
    network.roads.class[road as usize].rank()
}

pub fn movement_is_minor(network: &Network, node: u32, movement: &Movement) -> bool {
    let control = network.nodes.control[node as usize];
    if !matches!(control, Control::Stop | Control::Yield) {
        return false;
    }
    let highest = network
        .junction(node)
        .and_then(|junction| junction.movements.iter().map(|m| m.rank.0).max());
    highest.is_some_and(|top| movement.rank.0 < top)
}
