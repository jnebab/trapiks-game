use crate::consts::BOUNDARY_WEIGHT;
use crate::network::{LinkId, Network, road_of};
use crate::rng::Pcg32;
use crate::routing::{LinkCosts, RouteGraph};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeightedLinks {
    pub links: Vec<LinkId>,
    pub cumulative: Vec<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DemandTables {
    pub origins: WeightedLinks,
    pub destinations: WeightedLinks,
}

impl WeightedLinks {
    fn push(&mut self, link: LinkId, weight: f64) {
        if weight <= 0.0 {
            return;
        }
        let total = self.total();
        self.links.push(link);
        self.cumulative.push(total + weight);
    }

    pub fn total(&self) -> f64 {
        self.cumulative.last().copied().unwrap_or(0.0)
    }

    pub fn sample(&self, rng: &mut Pcg32) -> Option<LinkId> {
        if self.links.is_empty() {
            return None;
        }
        let x = rng.next_f64() * self.total();
        let index = self.cumulative.partition_point(|&c| c <= x);
        self.links.get(index).copied()
    }
}

impl DemandTables {
    pub fn build(network: &Network, graph: &RouteGraph, costs: &LinkCosts) -> DemandTables {
        let mut tables = DemandTables::default();
        let sources = network.region().map(|mask| mask.sources.as_slice());
        let sinks = network.region().map(|mask| mask.sinks.as_slice());
        let active = (0..network.link_count() as LinkId).filter(|&l| network.is_link_active(l));
        for link in active {
            let weight = base_weight(network, costs, link);
            if !graph.successors(link).is_empty() {
                tables.origins.push(link, weight * boost(sources, link));
            }
            if !graph.predecessors(link).is_empty() {
                tables.destinations.push(link, weight * boost(sinks, link));
            }
        }
        tables
    }
}

fn base_weight(network: &Network, costs: &LinkCosts, link: LinkId) -> f64 {
    let class = network.roads.class[road_of(link) as usize];
    let drive_len = costs.drive_len.get(link as usize).copied().unwrap_or(0.0);
    drive_len * class.demand_weight()
}

fn boost(boundary: Option<&[LinkId]>, link: LinkId) -> f64 {
    match boundary {
        Some(links) if links.binary_search(&link).is_ok() => BOUNDARY_WEIGHT,
        _ => 1.0,
    }
}
