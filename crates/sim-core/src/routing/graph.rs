use crate::consts::{LEFT_TURN_PENALTY, UTURN_PENALTY};
use crate::network::{LinkId, Network, TurnKind};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Succ {
    pub to: LinkId,
    pub penalty: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pred {
    pub from: LinkId,
    pub penalty: f32,
}

#[derive(Clone, Debug, Default)]
pub struct RouteGraph {
    succ_start: Vec<u32>,
    succ: Vec<Succ>,
    pred_start: Vec<u32>,
    pred: Vec<Pred>,
    built_version: u64,
}

type Edge = (LinkId, LinkId, f32);

impl RouteGraph {
    pub fn build(network: &Network) -> RouteGraph {
        let edges = collect_edges(network);
        let n = network.link_count();
        let (succ_start, mut succ) = csr(n, &edges, |&(from, to, penalty)| {
            (from, Succ { to, penalty })
        });
        let (pred_start, mut pred) = csr(n, &edges, |&(from, to, penalty)| {
            (to, Pred { from, penalty })
        });
        sort_segments(&succ_start, &mut succ, |s| s.to);
        sort_segments(&pred_start, &mut pred, |p| p.from);
        RouteGraph {
            succ_start,
            succ,
            pred_start,
            pred,
            built_version: network.version(),
        }
    }

    pub fn built_version(&self) -> u64 {
        self.built_version
    }

    pub fn link_count(&self) -> usize {
        self.succ_start.len().saturating_sub(1)
    }

    pub fn successors(&self, link: LinkId) -> &[Succ] {
        segment(&self.succ_start, &self.succ, link)
    }

    pub fn predecessors(&self, link: LinkId) -> &[Pred] {
        segment(&self.pred_start, &self.pred, link)
    }
}

fn penalty(kind: TurnKind) -> f32 {
    match kind {
        TurnKind::Left => LEFT_TURN_PENALTY as f32,
        TurnKind::UTurn => UTURN_PENALTY as f32,
        TurnKind::Right | TurnKind::Through => 0.0,
    }
}

fn collect_edges(network: &Network) -> Vec<Edge> {
    let mut edges = Vec::new();
    for node in 0..network.nodes.count() as u32 {
        let turns = network.turns(node);
        edges.extend(
            turns
                .into_iter()
                .map(|(from, to, kind)| (from, to, penalty(kind))),
        );
    }
    edges
}

fn csr<T: Copy + Default>(
    n: usize,
    edges: &[Edge],
    entry: impl Fn(&Edge) -> (LinkId, T),
) -> (Vec<u32>, Vec<T>) {
    let mut start = vec![0u32; n + 1];
    for edge in edges {
        if let Some(count) = start.get_mut(entry(edge).0 as usize + 1) {
            *count += 1;
        }
    }
    for i in 1..start.len() {
        start[i] += start[i - 1];
    }
    let mut fill: Vec<u32> = start.clone();
    let mut out = vec![T::default(); start[n] as usize];
    for edge in edges {
        let (key, value) = entry(edge);
        let key = key as usize;
        if key < n {
            out[fill[key] as usize] = value;
            fill[key] += 1;
        }
    }
    (start, out)
}

fn sort_segments<T>(start: &[u32], items: &mut [T], key: impl Fn(&T) -> LinkId) {
    for pair in start.windows(2) {
        items[pair[0] as usize..pair[1] as usize].sort_unstable_by_key(&key);
    }
}

fn segment<'a, T>(start: &[u32], items: &'a [T], link: LinkId) -> &'a [T] {
    let index = link as usize;
    match (start.get(index), start.get(index + 1)) {
        (Some(&a), Some(&b)) => &items[a as usize..b as usize],
        _ => &[],
    }
}
