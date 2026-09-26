use std::collections::BinaryHeap;

use crate::network::{LinkId, Network};

use super::costs::LinkCosts;
use super::graph::RouteGraph;
use super::heap::HeapItem;
use super::landmarks::Landmarks;
use super::target::Target;

const NO_PARENT: u32 = u32::MAX;
const QUANTUM_PER_SECOND: f64 = 1e3;

#[derive(Clone, Copy)]
pub struct RouteContext<'a> {
    pub graph: &'a RouteGraph,
    pub costs: &'a LinkCosts,
    pub landmarks: Option<&'a Landmarks>,
    pub network: &'a Network,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RouteStats {
    pub queries: u64,
    pub expanded: u64,
}

#[derive(Clone, Copy)]
struct Slot {
    g: f64,
    parent: u32,
    stamp: u32,
}

const EMPTY_SLOT: Slot = Slot {
    g: f64::INFINITY,
    parent: NO_PARENT,
    stamp: 0,
};

#[derive(Default)]
pub struct Router {
    slots: Vec<Slot>,
    epoch: u32,
    heap: BinaryHeap<HeapItem>,
    stats: RouteStats,
}

impl RouteContext<'_> {
    fn target(&self, link: LinkId) -> Target {
        match self.landmarks {
            Some(landmarks) => landmarks.target(self.network, link),
            None => Target::euclid_only(self.network, link),
        }
    }

    fn estimate(&self, v: LinkId, target: &Target) -> f64 {
        if target.is_goal(v) {
            return 0.0;
        }
        match self.landmarks {
            Some(landmarks) => landmarks.estimate(self.network, self.costs, v, target),
            None => target.euclid(self.network, self.costs, v),
        }
    }
}

impl Router {
    pub fn new() -> Router {
        Router {
            epoch: 1,
            ..Router::default()
        }
    }

    pub fn stats(&self) -> RouteStats {
        self.stats
    }

    pub fn route_into(
        &mut self,
        ctx: RouteContext,
        from: LinkId,
        to: LinkId,
        out: &mut Vec<LinkId>,
    ) -> bool {
        self.stats.queries += 1;
        out.clear();
        let n = ctx.graph.link_count();
        if !endpoints_ok(ctx.network, n, from, to) {
            return false;
        }
        self.begin(n);
        let target = ctx.target(to);
        self.open(from, 0.0, NO_PARENT, ctx.estimate(from, &target));
        while let Some(top) = self.heap.pop() {
            if top.g > self.slots[top.link as usize].g {
                continue;
            }
            self.stats.expanded += 1;
            if top.link == to {
                self.trace(to, out);
                return true;
            }
            self.expand(&ctx, top, &target);
        }
        false
    }

    fn begin(&mut self, n: usize) {
        self.slots.resize(n, EMPTY_SLOT);
        self.heap.clear();
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.slots.iter_mut().for_each(|slot| slot.stamp = 0);
            self.epoch = 1;
        }
    }

    fn expand(&mut self, ctx: &RouteContext, top: HeapItem, target: &Target) {
        for s in ctx.graph.successors(top.link) {
            let g = top.g + ctx.costs.travel_time(s.to) + f64::from(s.penalty);
            if self.improves(s.to, g) {
                self.open(s.to, g, top.link, ctx.estimate(s.to, target));
            }
        }
    }

    fn improves(&self, link: LinkId, g: f64) -> bool {
        let slot = &self.slots[link as usize];
        slot.stamp != self.epoch || g < slot.g
    }

    fn open(&mut self, link: LinkId, g: f64, parent: u32, h: f64) {
        self.slots[link as usize] = Slot {
            g,
            parent,
            stamp: self.epoch,
        };
        self.heap.push(HeapItem {
            f: quantise(g + h),
            g,
            link,
        });
    }

    fn trace(&self, to: LinkId, out: &mut Vec<LinkId>) {
        let mut link = to;
        out.push(link);
        while self.slots[link as usize].parent != NO_PARENT {
            link = self.slots[link as usize].parent;
            out.push(link);
        }
        out.reverse();
    }
}

fn endpoints_ok(network: &Network, n: usize, from: LinkId, to: LinkId) -> bool {
    (from as usize) < n
        && (to as usize) < n
        && network.is_link_active(from)
        && network.is_link_active(to)
}

fn quantise(f: f64) -> f64 {
    ((f * QUANTUM_PER_SECOND + 0.5) as u64) as f64
}
