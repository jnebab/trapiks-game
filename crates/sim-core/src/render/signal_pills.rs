use crate::network::{LinkId, Network, SignalState, road_of};

const PILL_GAP: f64 = 1.2;

pub const PILL_GREEN: u8 = 0;
pub const PILL_AMBER: u8 = 1;
pub const PILL_RED: u8 = 2;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SignalPills {
    pub link: Vec<u32>,
    pub x: Vec<f32>,
    pub y: Vec<f32>,
    pub angle: Vec<f32>,
}

pub fn signal_pills(network: &Network) -> SignalPills {
    let mut pills = SignalPills::default();
    for link in 0..network.link_count() as LinkId {
        if network.signal_state(link, 0).is_some() {
            add_pill(network, link, &mut pills);
        }
    }
    pills
}

fn add_pill(network: &Network, link: LinkId, pills: &mut SignalPills) {
    let (centre, dir) = network.centre_pose(link, network.link_span(link).1);
    let offset = network.roads.width(road_of(link)) / 2.0 + PILL_GAP;
    let at = centre + dir.perp_right() * offset;
    pills.link.push(link);
    pills.x.push(at.x as f32);
    pills.y.push(at.y as f32);
    pills.angle.push(libm::atan2(dir.y, dir.x) as f32);
}

pub fn signal_states(network: &Network, pills: &[u32], tick: u64, out: &mut Vec<u8>) {
    out.clear();
    out.extend(
        pills
            .iter()
            .map(|&link| state_code(network.signal_state(link, tick))),
    );
}

fn state_code(state: Option<SignalState>) -> u8 {
    match state {
        Some(SignalState::Green) => PILL_GREEN,
        Some(SignalState::Amber) => PILL_AMBER,
        Some(SignalState::Red) | None => PILL_RED,
    }
}
