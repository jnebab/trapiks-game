use crate::network::{Junction, LinkId, Movement, Network};

pub fn entry_lane(junction: &Junction, from_link: LinkId, to_link: LinkId) -> Option<u8> {
    let index = junction.movement_index(from_link, to_link)?;
    Some(junction.movements[index].from_lanes.0)
}

pub fn landing_lane(
    network: &Network,
    movement: &Movement,
    route: &[LinkId],
    target_index: usize,
) -> Option<u8> {
    let target = *route.get(target_index)?;
    match route.get(target_index + 1) {
        Some(&after) => entry_lane(network.junction(network.link_to(target))?, target, after),
        None => Some(
            movement
                .primary_lanes
                .1
                .min(network.link_lanes(target).saturating_sub(1)),
        ),
    }
}
