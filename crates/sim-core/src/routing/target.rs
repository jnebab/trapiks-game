use crate::consts::LANDMARK_COUNT;
use crate::geom::Vec2;
use crate::network::{LinkId, Network};

use super::costs::LinkCosts;
use super::landmarks::{LinkRow, Row, UNREACHED_FROM, UNREACHED_TO};

const ROUNDING_SLACK: f32 = 1.0 - 2.0 * f32::EPSILON;
const STRAIGHT_SLACK: f64 = 1.0 - 2.0 * f64::EPSILON;

pub struct Target {
    link: LinkId,
    point: Vec2,
    from_t: Row,
    to_t: Row,
}

impl Target {
    pub fn euclid_only(network: &Network, link: LinkId) -> Target {
        Target {
            link,
            point: from_point(network, link),
            from_t: [UNREACHED_TO; LANDMARK_COUNT],
            to_t: [UNREACHED_FROM; LANDMARK_COUNT],
        }
    }

    pub fn with_landmarks(network: &Network, link: LinkId, goal: &LinkRow) -> Target {
        Target {
            link,
            point: from_point(network, link),
            from_t: goal
                .from_l
                .map(|d| swap_unreached(d, UNREACHED_FROM, UNREACHED_TO)),
            to_t: goal
                .to_l
                .map(|d| swap_unreached(d, UNREACHED_TO, UNREACHED_FROM)),
        }
    }

    pub fn is_goal(&self, link: LinkId) -> bool {
        self.link == link
    }

    pub fn euclid(&self, network: &Network, costs: &LinkCosts, v: LinkId) -> f64 {
        straight(to_point(network, v), self.point, costs)
    }

    pub fn bound(&self, row: &LinkRow, costs: &LinkCosts) -> f64 {
        let pairs: Row = std::array::from_fn(|i| {
            (self.from_t[i] - row.from_l[i]).max(row.to_l[i] - self.to_t[i])
        });
        let alt = pairs.into_iter().fold(0.0, f32::max) * ROUNDING_SLACK;
        straight(row.end, self.point, costs).max(f64::from(alt))
    }
}

fn swap_unreached(d: f32, unreached: f32, replacement: f32) -> f32 {
    if d == unreached { replacement } else { d }
}

fn straight(a: Vec2, b: Vec2, costs: &LinkCosts) -> f64 {
    a.distance(b) * costs.inverse_heuristic_speed() * STRAIGHT_SLACK
}

pub fn to_point(network: &Network, link: LinkId) -> Vec2 {
    network.nodes.pos[network.link_to(link) as usize]
}

fn from_point(network: &Network, link: LinkId) -> Vec2 {
    network.nodes.pos[network.link_from(link) as usize]
}
