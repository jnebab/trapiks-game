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

    pub fn patch(&mut self, network: &Network, nodes: &[u32]) {
        let changed = node_mask(network, nodes);
        let mut edges: Vec<Edge> = nodes
            .iter()
            .flat_map(|&node| node_edges(network, node))
            .collect();
        edges.sort_unstable_by_key(|&(from, to, _)| (from, to));
        let (succ_start, succ) = splice(&self.succ_start, &self.succ, |link| {
            changed[network.link_to(link) as usize].then(|| {
                edges_matching(
                    &edges,
                    |e| e.0 == link,
                    |e| Succ {
                        to: e.1,
                        penalty: e.2,
                    },
                )
            })
        });
        edges.sort_unstable_by_key(|&(from, to, _)| (to, from));
        let (pred_start, pred) = splice(&self.pred_start, &self.pred, |link| {
            changed[network.link_from(link) as usize].then(|| {
                edges_matching(
                    &edges,
                    |e| e.1 == link,
                    |e| Pred {
                        from: e.0,
                        penalty: e.2,
                    },
                )
            })
        });
        *self = RouteGraph {
            succ_start,
            succ,
            pred_start,
            pred,
            built_version: network.version(),
        };
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

fn node_edges(network: &Network, node: u32) -> impl Iterator<Item = Edge> {
    network
        .turns(node)
        .into_iter()
        .map(|(from, to, kind)| (from, to, penalty(kind)))
}

fn collect_edges(network: &Network) -> Vec<Edge> {
    (0..network.nodes.count() as u32)
        .flat_map(|node| node_edges(network, node))
        .collect()
}

fn node_mask(network: &Network, nodes: &[u32]) -> Vec<bool> {
    let mut mask = vec![false; network.nodes.count()];
    for &node in nodes {
        if let Some(flag) = mask.get_mut(node as usize) {
            *flag = true;
        }
    }
    mask
}

fn edges_matching<T>(
    edges: &[Edge],
    keep: impl Fn(&Edge) -> bool,
    entry: impl Fn(&Edge) -> T,
) -> Vec<T> {
    edges.iter().filter(|e| keep(e)).map(entry).collect()
}

fn splice<T: Copy>(
    start: &[u32],
    items: &[T],
    replacement: impl Fn(LinkId) -> Option<Vec<T>>,
) -> (Vec<u32>, Vec<T>) {
    let mut out_start = Vec::with_capacity(start.len());
    let mut out = Vec::with_capacity(items.len());
    out_start.push(0);
    for link in 0..start.len().saturating_sub(1) as LinkId {
        match replacement(link) {
            Some(fresh) => out.extend_from_slice(&fresh),
            None => out.extend_from_slice(segment(start, items, link)),
        }
        out_start.push(out.len() as u32);
    }
    (out_start, out)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{GridCity, grid_city};

    fn assert_same(a: &RouteGraph, b: &RouteGraph) {
        for link in 0..a.link_count() as LinkId {
            assert_eq!(a.successors(link), b.successors(link), "succ {link}");
            assert_eq!(a.predecessors(link), b.predecessors(link), "pred {link}");
        }
    }

    #[test]
    fn patch_matches_full_build_after_delete_and_restore() {
        let map = grid_city(&GridCity {
            cols: 4,
            rows: 4,
            spacing: 100.0,
        });
        let mut network = Network::from_map(&map);
        let mut graph = RouteGraph::build(&network);
        for deleted in [true, false] {
            let built = graph.built_version();
            network.set_road_deleted(5, deleted);
            let ends = [network.roads.from[5], network.roads.to[5]];
            network.commit_edit(&ends, false);
            let changes = network.changes_since(built);
            let nodes = changes.map(|c| c.nodes).unwrap_or_default();
            assert_eq!(nodes, ends.to_vec());
            graph.patch(&network, &nodes);
            assert_same(&graph, &RouteGraph::build(&network));
        }
    }
}
