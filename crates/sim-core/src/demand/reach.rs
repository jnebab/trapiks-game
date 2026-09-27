use crate::network::{LinkId, Network};
use crate::routing::{RouteGraph, largest_component_members, reachable_from, reaching};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reach {
    pub can_leave: Vec<bool>,
    pub can_arrive: Vec<bool>,
}

impl Reach {
    pub fn build(network: &Network, graph: &RouteGraph) -> Reach {
        let mask = active_mask(network, graph);
        let active = |link: LinkId| mask[link as usize];
        let core = largest_component_members(graph, active);
        Reach {
            can_leave: reaching(graph, &core, active),
            can_arrive: reachable_from(graph, &core, active),
        }
    }

    pub fn update(&self, network: &Network, graph: &RouteGraph) -> Reach {
        let mask = active_mask(network, graph);
        let active = |link: LinkId| mask[link as usize];
        let Some(seed) = self.core_seed(&mask) else {
            return Reach::build(network, graph);
        };
        let mut sources = vec![false; mask.len()];
        sources[seed as usize] = true;
        let reach = Reach {
            can_leave: reaching(graph, &sources, active),
            can_arrive: reachable_from(graph, &sources, active),
        };
        if reach.is_majority(&mask) {
            return reach;
        }
        Reach::build(network, graph)
    }

    fn is_majority(&self, mask: &[bool]) -> bool {
        let active_count = mask.iter().filter(|&&a| a).count();
        self.core_size() * 2 > active_count
    }

    fn core_seed(&self, mask: &[bool]) -> Option<LinkId> {
        (0..mask.len() as LinkId)
            .find(|&link| mask[link as usize] && self.origin_ok(link) && self.destination_ok(link))
    }

    fn core_size(&self) -> usize {
        self.can_leave
            .iter()
            .zip(&self.can_arrive)
            .filter(|&(&leave, &arrive)| leave && arrive)
            .count()
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

fn active_mask(network: &Network, graph: &RouteGraph) -> Vec<bool> {
    (0..graph.link_count() as LinkId)
        .map(|link| network.is_link_active(link))
        .collect()
}
