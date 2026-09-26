use std::collections::BinaryHeap;

use crate::network::LinkId;

use super::costs::LinkCosts;
use super::graph::RouteGraph;
use super::heap::HeapItem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sweep {
    Forward,
    Backward,
}

pub struct Search {
    pub dist: Vec<f64>,
    settled: Vec<u32>,
    epoch: u32,
    heap: BinaryHeap<HeapItem>,
}

impl Search {
    pub fn new() -> Search {
        Search {
            dist: Vec::new(),
            settled: Vec::new(),
            epoch: 0,
            heap: BinaryHeap::new(),
        }
    }

    pub fn run(
        &mut self,
        graph: &RouteGraph,
        start: LinkId,
        sweep: Sweep,
        cost: impl Fn(LinkId) -> f64,
    ) {
        self.reset(graph.link_count());
        let Some(slot) = self.dist.get_mut(start as usize) else {
            return;
        };
        *slot = 0.0;
        self.heap.push(item(0.0, start));
        while let Some(top) = self.heap.pop() {
            if self.settle(top) {
                self.relax(graph, top, sweep, &cost);
            }
        }
    }

    fn reset(&mut self, n: usize) {
        self.dist.clear();
        self.dist.resize(n, f64::INFINITY);
        self.settled.resize(n, 0);
        self.heap.clear();
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.settled.iter_mut().for_each(|stamp| *stamp = 0);
            self.epoch = 1;
        }
    }

    fn settle(&mut self, top: HeapItem) -> bool {
        let index = top.link as usize;
        if self.settled[index] == self.epoch || top.g > self.dist[index] {
            return false;
        }
        self.settled[index] = self.epoch;
        true
    }

    fn relax(
        &mut self,
        graph: &RouteGraph,
        top: HeapItem,
        sweep: Sweep,
        cost: &impl Fn(LinkId) -> f64,
    ) {
        match sweep {
            Sweep::Forward => {
                for s in graph.successors(top.link) {
                    self.offer(s.to, top.g + cost(s.to) + f64::from(s.penalty));
                }
            }
            Sweep::Backward => {
                let base = top.g + cost(top.link);
                for p in graph.predecessors(top.link) {
                    self.offer(p.from, base + f64::from(p.penalty));
                }
            }
        }
    }

    fn offer(&mut self, link: LinkId, g: f64) {
        let index = link as usize;
        if g < self.dist[index] {
            self.dist[index] = g;
            self.heap.push(item(g, link));
        }
    }
}

impl Default for Search {
    fn default() -> Self {
        Search::new()
    }
}

fn item(g: f64, link: LinkId) -> HeapItem {
    HeapItem { f: g, g, link }
}

pub fn dijkstra_cost(
    graph: &RouteGraph,
    costs: &LinkCosts,
    from: LinkId,
    to: LinkId,
) -> Option<f64> {
    let mut search = Search::new();
    search.run(graph, from, Sweep::Forward, |link| costs.travel_time(link));
    search
        .dist
        .get(to as usize)
        .copied()
        .filter(|d| d.is_finite())
}
