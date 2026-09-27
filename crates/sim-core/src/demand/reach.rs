use crate::network::{LinkId, Network};
use crate::routing::{RouteGraph, largest_component_members, reachable_from, reaching};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reach {
    pub can_leave: Vec<bool>,
    pub can_arrive: Vec<bool>,
}

impl Reach {
    pub fn build(network: &Network, graph: &RouteGraph) -> Reach {
        let mask: Vec<bool> = (0..graph.link_count() as LinkId)
            .map(|link| network.is_link_active(link))
            .collect();
        let active = |link: LinkId| mask[link as usize];
        let core = largest_component_members(graph, active);
        Reach {
            can_leave: reaching(graph, &core, active),
            can_arrive: reachable_from(graph, &core, active),
        }
    }

    pub fn origin_ok(&self, link: LinkId) -> bool {
        self.can_leave.get(link as usize).copied().unwrap_or(false)
    }

    pub fn destination_ok(&self, link: LinkId) -> bool {
        self.can_arrive.get(link as usize).copied().unwrap_or(false)
    }

    pub fn changed_links(&self, other: &Reach) -> Vec<LinkId> {
        let count = self.can_leave.len().max(other.can_leave.len());
        (0..count as LinkId)
            .filter(|&link| {
                self.origin_ok(link) != other.origin_ok(link)
                    || self.destination_ok(link) != other.destination_ok(link)
            })
            .collect()
    }
}
