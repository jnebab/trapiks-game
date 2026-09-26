use crate::network::{LinkId, Movement, Network};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Landing {
    Fixed(u8),
    Clamp { low: u8, high: u8, top: u8 },
}

impl Landing {
    pub fn lane(self, from_lane: u8) -> u8 {
        match self {
            Landing::Fixed(lane) => lane,
            Landing::Clamp { low, high, top } => from_lane.clamp(low, high.max(low)).min(top),
        }
    }
}

pub fn movement_lanes(network: &Network, from: LinkId, to: LinkId) -> Option<(u8, u8)> {
    let junction = network.junction(network.link_to(from))?;
    let index = junction.movement_index(from, to)?;
    Some(junction.movements[index].from_lanes)
}

pub fn allowed_lanes(network: &Network, route: &[LinkId], cursor: usize) -> Option<(u8, u8)> {
    let link = *route.get(cursor)?;
    match route.get(cursor + 1) {
        Some(&next) => movement_lanes(network, link, next),
        None => Some((0, network.link_lanes(link).saturating_sub(1))),
    }
}

pub fn landing_rule(
    network: &Network,
    movement: &Movement,
    route: &[LinkId],
    target_index: usize,
) -> Option<Landing> {
    let target = *route.get(target_index)?;
    let top = network.link_lanes(target).saturating_sub(1);
    match route.get(target_index + 1) {
        Some(&after) => {
            let (low, high) = movement_lanes(network, target, after)?;
            Some(Landing::Clamp { low, high, top })
        }
        None => Some(Landing::Fixed(movement.primary_lanes.1.min(top))),
    }
}

pub fn landing_lane(
    network: &Network,
    movement: &Movement,
    route: &[LinkId],
    target_index: usize,
    from_lane: u8,
) -> Option<u8> {
    Some(landing_rule(network, movement, route, target_index)?.lane(from_lane))
}
