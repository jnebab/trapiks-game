use crate::consts::LANDMARK_COUNT;
use crate::geom::Vec2;
use crate::network::{LinkId, Network};

use super::costs::LinkCosts;
use super::dijkstra::{Search, Sweep};
use super::graph::RouteGraph;

const MAX_SEEDS: usize = 4;

#[derive(Clone, Debug, Default)]
pub struct Landmarks {
    links: Vec<LinkId>,
    from_l: Vec<Row>,
    to_l: Vec<Row>,
}

type Row = [f32; LANDMARK_COUNT];

const EMPTY_ROW: Row = [f32::INFINITY; LANDMARK_COUNT];

pub struct Target {
    link: LinkId,
    point: Vec2,
    from_t: Row,
    to_t: Row,
}

struct Builder<'a> {
    graph: &'a RouteGraph,
    costs: &'a LinkCosts,
    search: Search,
}

impl Landmarks {
    pub fn build(network: &Network, graph: &RouteGraph, costs: &LinkCosts) -> Landmarks {
        let mut builder = Builder {
            graph,
            costs,
            search: Search::new(),
        };
        let reach = builder.best_seed(network);
        builder.select(&reach)
    }

    pub fn count(&self) -> usize {
        self.links.len()
    }

    pub fn target(&self, network: &Network, link: LinkId) -> Target {
        Target {
            link,
            point: from_point(network, link),
            from_t: row(&self.from_l, link),
            to_t: row(&self.to_l, link),
        }
    }

    pub fn estimate(&self, network: &Network, costs: &LinkCosts, v: LinkId, t: &Target) -> f64 {
        let mut best = t.euclid(network, costs, v).max(0.0);
        let from_v = row(&self.from_l, v);
        let to_v = row(&self.to_l, v);
        for i in 0..self.links.len() {
            best = best.max(difference(t.from_t[i], from_v[i]));
            best = best.max(difference(to_v[i], t.to_t[i]));
        }
        best
    }

    fn push(&mut self, link: LinkId, forward: &[f64], backward: &[f64]) {
        let column = self.links.len();
        self.links.push(link);
        fill_column(&mut self.from_l, column, forward);
        fill_column(&mut self.to_l, column, backward);
    }
}

impl Target {
    pub fn is_goal(&self, link: LinkId) -> bool {
        self.link == link
    }

    pub fn euclid_only(network: &Network, link: LinkId) -> Target {
        Target {
            link,
            point: from_point(network, link),
            from_t: EMPTY_ROW,
            to_t: EMPTY_ROW,
        }
    }

    pub fn euclid(&self, network: &Network, costs: &LinkCosts, v: LinkId) -> f64 {
        to_point(network, v).distance(self.point) / costs.heuristic_speed()
    }
}

fn row(table: &[Row], link: LinkId) -> Row {
    table.get(link as usize).copied().unwrap_or(EMPTY_ROW)
}

fn fill_column(table: &mut Vec<Row>, column: usize, dist: &[f64]) {
    table.resize(dist.len(), EMPTY_ROW);
    for (row, &d) in table.iter_mut().zip(dist) {
        row[column] = d as f32;
    }
}

fn difference(a: f32, b: f32) -> f64 {
    if a.is_infinite() || b.is_infinite() {
        return 0.0;
    }
    f64::from(a) - f64::from(b)
}

fn to_point(network: &Network, link: LinkId) -> Vec2 {
    network.nodes.pos[network.link_to(link) as usize]
}

fn from_point(network: &Network, link: LinkId) -> Vec2 {
    network.nodes.pos[network.link_from(link) as usize]
}

impl Builder<'_> {
    fn forward(&mut self, start: LinkId) -> Vec<f64> {
        let costs = self.costs;
        self.search.run(self.graph, start, Sweep::Forward, |link| {
            costs.free_time(link)
        });
        self.search.dist.clone()
    }

    fn backward(&mut self, start: LinkId) -> Vec<f64> {
        let costs = self.costs;
        self.search.run(self.graph, start, Sweep::Backward, |link| {
            costs.free_time(link)
        });
        self.search.dist.clone()
    }

    fn best_seed(&mut self, network: &Network) -> Vec<f64> {
        let active: Vec<bool> = (0..self.graph.link_count() as LinkId)
            .map(|link| network.is_link_active(link))
            .collect();
        let active_count = active.iter().filter(|&&a| a).count();
        let mut reached = vec![false; active.len()];
        let mut best: Vec<f64> = Vec::new();
        let mut best_count = 0;
        for _ in 0..MAX_SEEDS {
            let Some(seed) = (0..active.len()).find(|&i| active[i] && !reached[i]) else {
                break;
            };
            let dist = self.forward(seed as LinkId);
            let count = mark_reached(&dist, &active, &mut reached);
            if count > best_count {
                best_count = count;
                best = dist;
            }
            if best_count * 2 >= active_count {
                break;
            }
        }
        best
    }

    fn select(mut self, reach: &[f64]) -> Landmarks {
        let mut landmarks = Landmarks::default();
        let mut nearest: Vec<f64> = reach.to_vec();
        while landmarks.count() < LANDMARK_COUNT {
            let Some(next) = farthest(reach, &nearest, &landmarks.links) else {
                break;
            };
            let forward = self.forward(next);
            tighten(&mut nearest, &forward, landmarks.links.is_empty());
            let backward = self.backward(next);
            landmarks.push(next, &forward, &backward);
        }
        landmarks
    }
}

fn tighten(nearest: &mut [f64], forward: &[f64], first: bool) {
    if first {
        nearest.clone_from_slice(forward);
        return;
    }
    for (near, &d) in nearest.iter_mut().zip(forward) {
        *near = near.min(d);
    }
}

fn mark_reached(dist: &[f64], active: &[bool], reached: &mut [bool]) -> usize {
    let mut count = 0;
    for (i, d) in dist.iter().enumerate() {
        if d.is_finite() && active[i] {
            reached[i] = true;
            count += 1;
        }
    }
    count
}

fn farthest(reach: &[f64], nearest: &[f64], chosen: &[LinkId]) -> Option<LinkId> {
    let mut best: Option<(f64, LinkId)> = None;
    for (i, (&r, &d)) in reach.iter().zip(nearest).enumerate() {
        let link = i as LinkId;
        if !r.is_finite() || !d.is_finite() || chosen.contains(&link) {
            continue;
        }
        if best.is_none_or(|(top, _)| d > top) {
            best = Some((d, link));
        }
    }
    best.map(|(_, link)| link)
}
