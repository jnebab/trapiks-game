use std::collections::BTreeSet;

use trapiks_sim_core::geo::Projection;
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::MapData;
use trapiks_sim_core::network::Network;

use crate::expected::names_match;

pub const CLUSTER_RADIUS_M: f64 = 150.0;

pub struct Site {
    pub node: u32,
    pub distance: f64,
    pub names: Vec<String>,
    pub lat: f64,
    pub lon: f64,
}

pub struct Locator<'a> {
    pub map: &'a MapData,
    pub network: &'a Network,
    pub projection: Projection,
    pub junctions: Vec<u32>,
}

impl<'a> Locator<'a> {
    pub fn new(map: &'a MapData, network: &'a Network) -> Locator<'a> {
        let junctions = (0..network.nodes.count() as u32)
            .filter(|&node| network.active_degree(node) >= 3)
            .collect();
        Locator {
            map,
            network,
            projection: Projection::from_origin(map.origin.clone()),
            junctions,
        }
    }

    fn pos(&self, node: u32) -> Vec2 {
        self.network.nodes.pos[node as usize]
    }

    pub fn within(&self, center: Vec2, radius: f64) -> Vec<u32> {
        self.junctions
            .iter()
            .copied()
            .filter(|&node| self.pos(node).distance(center) <= radius)
            .collect()
    }

    pub fn nearest(&self, center: Vec2) -> Option<Site> {
        let node = self.junctions.iter().copied().min_by(|&a, &b| {
            self.pos(a)
                .distance(center)
                .total_cmp(&self.pos(b).distance(center))
        })?;
        Some(self.site(node, center, self.road_names(&[node])))
    }

    pub fn cluster_names(&self, center: Vec2) -> Vec<String> {
        self.road_names(&self.within(center, CLUSTER_RADIUS_M))
    }

    pub fn best_match(&self, center: Vec2, radius: f64, expected: &[&[&str]]) -> Option<Site> {
        let mut candidates = self.within(center, radius);
        candidates.sort_by(|&a, &b| {
            let (da, db) = (self.pos(a).distance(center), self.pos(b).distance(center));
            da.total_cmp(&db).then(a.cmp(&b))
        });
        candidates.into_iter().find_map(|node| {
            let names = self.cluster_names(self.pos(node));
            names_match(expected, &names).then(|| self.site(node, center, names))
        })
    }

    pub fn busiest_in_cluster(&self, node: u32) -> u32 {
        self.within(self.pos(node), CLUSTER_RADIUS_M)
            .into_iter()
            .max_by_key(|&n| (self.incident_lanes(n), std::cmp::Reverse(n)))
            .unwrap_or(node)
    }

    fn incident_lanes(&self, node: u32) -> u32 {
        self.active_roads(&[node])
            .map(|road| u32::from(self.network.roads.total_lanes(road)))
            .sum()
    }

    fn active_roads<'b>(&'b self, nodes: &'b [u32]) -> impl Iterator<Item = u32> + 'b {
        nodes
            .iter()
            .flat_map(|&node| self.network.nodes.roads[node as usize].iter().copied())
            .filter(|&road| self.network.is_road_active(road))
    }

    pub fn road_names(&self, nodes: &[u32]) -> Vec<String> {
        let names: BTreeSet<String> = self
            .active_roads(nodes)
            .filter_map(|road| self.map.roads.name.get(road as usize))
            .filter_map(|&index| self.map.names.get(index as usize))
            .filter(|name| !name.is_empty())
            .cloned()
            .collect();
        names.into_iter().collect()
    }

    fn site(&self, node: u32, center: Vec2, names: Vec<String>) -> Site {
        let pos = self.pos(node);
        let (lat, lon) = self.projection.unproject(pos.x, pos.y);
        Site {
            node,
            distance: pos.distance(center),
            names,
            lat,
            lon,
        }
    }
}
