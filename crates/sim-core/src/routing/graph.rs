use crate::consts::{LEFT_TURN_PENALTY, UTURN_PENALTY};
use crate::network::{LinkId, Network, TurnKind, reverse};

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
        let n = self.link_count();
        let mut edges: Vec<Edge> = nodes
            .iter()
            .flat_map(|&node| node_edges(network, node))
            .collect();
        edges.sort_unstable_by_key(|&(from, to, _)| (from, to));
        let arriving = touching(network, nodes, n, |link| network.link_to(link));
        let fresh_succ = replacements(
            &arriving,
            &edges,
            |e| e.0,
            |e| Succ {
                to: e.1,
                penalty: e.2,
            },
        );
        edges.sort_unstable_by_key(|&(from, to, _)| (to, from));
        let departing = touching(network, nodes, n, |link| network.link_from(link));
        let fresh_pred = replacements(
            &departing,
            &edges,
            |e| e.1,
            |e| Pred {
                from: e.0,
                penalty: e.2,
            },
        );
        splice(&mut self.succ_start, &mut self.succ, &fresh_succ);
        splice(&mut self.pred_start, &mut self.pred, &fresh_pred);
        self.built_version = network.version();
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

fn touching(
    network: &Network,
    nodes: &[u32],
    n: usize,
    end_of: impl Fn(LinkId) -> u32,
) -> Vec<LinkId> {
    let mut links: Vec<LinkId> = Vec::new();
    for &node in nodes {
        let Some(roads) = network.nodes.roads.get(node as usize) else {
            continue;
        };
        for &road in roads {
            let link = network.departing_link(road, node);
            links.extend(
                [link, reverse(link)]
                    .into_iter()
                    .filter(|&l| end_of(l) == node),
            );
        }
    }
    links.retain(|&link| (link as usize) < n);
    links.sort_unstable();
    links.dedup();
    links
}

fn replacements<T>(
    links: &[LinkId],
    edges: &[Edge],
    key: impl Fn(&Edge) -> LinkId,
    entry: impl Fn(&Edge) -> T,
) -> Vec<(LinkId, Vec<T>)> {
    links
        .iter()
        .map(|&link| {
            let first = edges.partition_point(|e| key(e) < link);
            let last = edges.partition_point(|e| key(e) <= link);
            (link, edges[first..last].iter().map(&entry).collect())
        })
        .collect()
}

fn splice<T: Copy + PartialEq>(start: &mut [u32], items: &mut Vec<T>, fresh: &[(LinkId, Vec<T>)]) {
    for (link, entries) in fresh.iter().rev() {
        let link = *link as usize;
        let (a, b) = (start[link] as usize, start[link + 1] as usize);
        if items[a..b] == entries[..] {
            continue;
        }
        items.splice(a..b, entries.iter().copied());
        let shift = entries.len() as i64 - (b - a) as i64;
        for s in &mut start[link + 1..] {
            *s = (i64::from(*s) + shift) as u32;
        }
    }
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
