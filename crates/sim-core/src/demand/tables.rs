use crate::consts::BOUNDARY_WEIGHT;
use crate::network::{LinkId, Network, road_of};
use crate::rng::Pcg32;
use crate::routing::{LinkCosts, RouteGraph};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeightedLinks {
    pub links: Vec<LinkId>,
    pub cumulative: Vec<f64>,
    weights: Vec<f64>,
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
        self.weights.push(weight);
        self.cumulative.push(total + weight);
    }

    fn set(&mut self, link: LinkId, weight: f64) {
        match (self.links.binary_search(&link), weight > 0.0) {
            (Ok(at), true) => self.weights[at] = weight,
            (Ok(at), false) => {
                self.links.remove(at);
                self.weights.remove(at);
            }
            (Err(at), true) => {
                self.links.insert(at, link);
                self.weights.insert(at, weight);
            }
            (Err(_), false) => {}
        }
    }

    fn accumulate(&mut self) {
        self.cumulative.clear();
        let mut total = 0.0;
        for &weight in &self.weights {
            total += weight;
            self.cumulative.push(total);
        }
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
        for link in 0..network.link_count() as LinkId {
            let (origin, destination) = link_weights(network, graph, costs, link);
            tables.origins.push(link, origin);
            tables.destinations.push(link, destination);
        }
        tables
    }

    pub fn update(
        &mut self,
        network: &Network,
        graph: &RouteGraph,
        costs: &LinkCosts,
        links: &[LinkId],
    ) {
        for &link in links {
            let (origin, destination) = link_weights(network, graph, costs, link);
            self.origins.set(link, origin);
            self.destinations.set(link, destination);
        }
        self.origins.accumulate();
        self.destinations.accumulate();
    }
}

fn link_weights(
    network: &Network,
    graph: &RouteGraph,
    costs: &LinkCosts,
    link: LinkId,
) -> (f64, f64) {
    if !network.is_link_active(link) {
        return (0.0, 0.0);
    }
    let sources = network.region().map(|mask| mask.sources.as_slice());
    let sinks = network.region().map(|mask| mask.sinks.as_slice());
    let weight = base_weight(network, costs, link);
    let origin = if graph.successors(link).is_empty() {
        0.0
    } else {
        weight * boost(sources, link)
    };
    let destination = if graph.predecessors(link).is_empty() {
        0.0
    } else {
        weight * boost(sinks, link)
    };
    (origin, destination)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{GridCity, grid_city};

    #[test]
    fn update_matches_full_build_after_delete() {
        let map = grid_city(&GridCity {
            cols: 4,
            rows: 4,
            spacing: 100.0,
        });
        let mut network = Network::from_map(&map);
        let mut graph = RouteGraph::build(&network);
        let mut costs = LinkCosts::build(&network);
        let mut tables = DemandTables::build(&network, &graph, &costs);
        let built = graph.built_version();
        network.set_road_deleted(5, true);
        let ends = [network.roads.from[5], network.roads.to[5]];
        network.commit_edit(&ends, false);
        let changes = network.changes_since(built).unwrap_or_default();
        graph.patch(&network, &changes.nodes);
        costs.refresh_links(&network, &changes.links);
        tables.update(&network, &graph, &costs, &changes.links);
        assert_eq!(tables, DemandTables::build(&network, &graph, &costs));
    }
}
