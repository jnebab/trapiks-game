use crate::consts::LANDMARK_COUNT;
use crate::geom::Vec2;
use crate::network::{LinkId, Network};

use super::costs::LinkCosts;
use super::dijkstra::{Search, Sweep};
use super::graph::RouteGraph;
use super::target::{Target, to_point};

const MAX_SEEDS: usize = 4;

pub type Row = [f32; LANDMARK_COUNT];

pub const UNREACHED_FROM: f32 = f32::INFINITY;
pub const UNREACHED_TO: f32 = f32::NEG_INFINITY;

#[derive(Clone, Copy, Debug)]
pub struct LinkRow {
    pub from_l: Row,
    pub to_l: Row,
    pub end: Vec2,
}

const EMPTY_LINK_ROW: LinkRow = LinkRow {
    from_l: [UNREACHED_FROM; LANDMARK_COUNT],
    to_l: [UNREACHED_TO; LANDMARK_COUNT],
    end: Vec2::new(0.0, 0.0),
};

#[derive(Clone, Debug, Default)]
pub struct Landmarks {
    links: Vec<LinkId>,
    rows: Vec<LinkRow>,
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
        let mut landmarks = builder.select(&reach);
        landmarks.fill_ends(network);
        landmarks
    }

    pub fn count(&self) -> usize {
        self.links.len()
    }

    pub fn links(&self) -> &[LinkId] {
        &self.links
    }

    pub fn target(&self, network: &Network, link: LinkId) -> Target {
        match self.rows.get(link as usize) {
            Some(row) => Target::with_landmarks(network, link, row),
            None => Target::euclid_only(network, link),
        }
    }

    pub fn estimate(&self, network: &Network, costs: &LinkCosts, v: LinkId, t: &Target) -> f64 {
        match self.rows.get(v as usize) {
            Some(row) => t.bound(row, costs),
            None => t.euclid(network, costs, v),
        }
    }

    fn fill_ends(&mut self, network: &Network) {
        for (link, row) in self.rows.iter_mut().enumerate() {
            row.end = to_point(network, link as LinkId);
        }
    }

    fn push(&mut self, link: LinkId, forward: &[f64], backward: &[f64]) {
        let column = self.links.len();
        self.links.push(link);
        self.rows.resize(forward.len(), EMPTY_LINK_ROW);
        for ((row, &f), &b) in self.rows.iter_mut().zip(forward).zip(backward) {
            row.from_l[column] = distance_or(f, UNREACHED_FROM);
            row.to_l[column] = distance_or(b, UNREACHED_TO);
        }
    }
}

fn distance_or(d: f64, unreached: f32) -> f32 {
    if d.is_finite() { d as f32 } else { unreached }
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
        let candidate = self.candidates(reach);
        let mut nearest: Vec<f64> = reach.to_vec();
        while landmarks.count() < LANDMARK_COUNT {
            let Some(next) = farthest(&candidate, &nearest, &landmarks.links) else {
                break;
            };
            let forward = self.forward(next);
            let backward = self.backward(next);
            tighten(
                &mut nearest,
                &forward,
                &backward,
                landmarks.links.is_empty(),
            );
            landmarks.push(next, &forward, &backward);
        }
        landmarks
    }

    fn candidates(&self, reach: &[f64]) -> Vec<bool> {
        reach
            .iter()
            .enumerate()
            .map(|(i, r)| r.is_finite() && self.connected(i as LinkId))
            .collect()
    }

    fn connected(&self, link: LinkId) -> bool {
        !self.graph.successors(link).is_empty() && !self.graph.predecessors(link).is_empty()
    }
}

fn tighten(nearest: &mut [f64], forward: &[f64], backward: &[f64], first: bool) {
    for ((near, &f), &b) in nearest.iter_mut().zip(forward).zip(backward) {
        let d = f.min(b);
        *near = if first { d } else { near.min(d) };
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

fn farthest(candidate: &[bool], nearest: &[f64], chosen: &[LinkId]) -> Option<LinkId> {
    let mut best: Option<(f64, LinkId)> = None;
    for (i, (&ok, &d)) in candidate.iter().zip(nearest).enumerate() {
        let link = i as LinkId;
        if !ok || !d.is_finite() || chosen.contains(&link) {
            continue;
        }
        if best.is_none_or(|(top, _)| d > top) {
            best = Some((d, link));
        }
    }
    best.map(|(_, link)| link)
}
