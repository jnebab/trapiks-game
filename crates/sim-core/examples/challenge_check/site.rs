use trapiks_sim_core::geo::Projection;
use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::map::MapData;
use trapiks_sim_core::network::Network;

use crate::expected::names_match;

pub const CANDIDATE_RADIUS_M: f64 = 600.0;
pub const MAX_CENTER_OFFSET_M: f64 = 150.0;

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
}

impl Locator<'_> {
    pub fn nearest(&self, center: Vec2) -> Option<Site> {
        self.closest(center, f64::INFINITY, |_| true)
    }

    pub fn best_match(&self, center: Vec2, expected: &[&[&str]]) -> Option<Site> {
        self.closest(center, CANDIDATE_RADIUS_M, |names| {
            names_match(expected, names)
        })
    }

    fn closest(
        &self,
        center: Vec2,
        radius: f64,
        accept: impl Fn(&[String]) -> bool,
    ) -> Option<Site> {
        let mut best: Option<Site> = None;
        for node in self.junction_nodes() {
            let distance = self.network.nodes.pos[node as usize].distance(center);
            let closer = best.as_ref().is_none_or(|site| distance < site.distance);
            if distance > radius || !closer {
                continue;
            }
            let names = self.road_names(node);
            if accept(&names) {
                best = Some(self.site(node, distance, names));
            }
        }
        best
    }

    fn junction_nodes(&self) -> impl Iterator<Item = u32> + '_ {
        (0..self.network.nodes.count() as u32).filter(|&node| self.network.active_degree(node) >= 3)
    }

    fn road_names(&self, node: u32) -> Vec<String> {
        let mut names: Vec<String> = self.network.nodes.roads[node as usize]
            .iter()
            .filter(|&&road| self.network.is_road_active(road))
            .filter_map(|&road| self.map.roads.name.get(road as usize))
            .filter_map(|&index| self.map.names.get(index as usize))
            .filter(|name| !name.is_empty())
            .cloned()
            .collect();
        names.sort();
        names.dedup();
        names
    }

    fn site(&self, node: u32, distance: f64, names: Vec<String>) -> Site {
        let pos = self.network.nodes.pos[node as usize];
        let (lat, lon) = self.projection.unproject(pos.x, pos.y);
        Site {
            node,
            distance,
            names,
            lat,
            lon,
        }
    }
}
