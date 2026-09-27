use crate::network::LinkId;

use super::graph::RouteGraph;

pub fn largest_component(graph: &RouteGraph, active: impl Fn(LinkId) -> bool) -> usize {
    largest_component_members(graph, active)
        .iter()
        .filter(|&&member| member)
        .count()
}

pub fn largest_component_members(graph: &RouteGraph, active: impl Fn(LinkId) -> bool) -> Vec<bool> {
    let order = finish_order(graph, &active);
    let mut assigned = vec![false; graph.link_count()];
    let mut largest: Vec<LinkId> = Vec::new();
    for &link in order.iter().rev() {
        if assigned[link as usize] {
            continue;
        }
        let component = collect_backward(graph, &active, link, &mut assigned);
        if component.len() > largest.len() {
            largest = component;
        }
    }
    let mut members = vec![false; graph.link_count()];
    for link in largest {
        members[link as usize] = true;
    }
    members
}

pub fn reaching(
    graph: &RouteGraph,
    targets: &[bool],
    active: impl Fn(LinkId) -> bool,
) -> Vec<bool> {
    let mut seen = targets.to_vec();
    let mut stack = seeds(targets);
    while let Some(link) = stack.pop() {
        for pred in graph.predecessors(link) {
            visit(pred.from, &active, &mut seen, &mut stack);
        }
    }
    seen
}

pub fn reachable_from(
    graph: &RouteGraph,
    sources: &[bool],
    active: impl Fn(LinkId) -> bool,
) -> Vec<bool> {
    let mut seen = sources.to_vec();
    let mut stack = seeds(sources);
    while let Some(link) = stack.pop() {
        for succ in graph.successors(link) {
            visit(succ.to, &active, &mut seen, &mut stack);
        }
    }
    seen
}

fn seeds(mask: &[bool]) -> Vec<LinkId> {
    (0..mask.len() as LinkId)
        .filter(|&link| mask[link as usize])
        .collect()
}

fn visit(
    link: LinkId,
    active: &impl Fn(LinkId) -> bool,
    seen: &mut [bool],
    stack: &mut Vec<LinkId>,
) {
    if !seen[link as usize] && active(link) {
        seen[link as usize] = true;
        stack.push(link);
    }
}

fn finish_order(graph: &RouteGraph, active: &impl Fn(LinkId) -> bool) -> Vec<LinkId> {
    let count = graph.link_count();
    let mut visited = vec![false; count];
    let mut order = Vec::with_capacity(count);
    for link in 0..count as LinkId {
        if active(link) && !visited[link as usize] {
            visit_forward(graph, active, link, &mut visited, &mut order);
        }
    }
    order
}

fn visit_forward(
    graph: &RouteGraph,
    active: &impl Fn(LinkId) -> bool,
    start: LinkId,
    visited: &mut [bool],
    order: &mut Vec<LinkId>,
) {
    visited[start as usize] = true;
    let mut stack = vec![(start, 0usize)];
    while let Some(top) = stack.last_mut() {
        let (link, next) = *top;
        top.1 += 1;
        match graph.successors(link).get(next) {
            Some(succ) => push_unvisited(active, succ.to, visited, &mut stack),
            None => {
                order.push(link);
                stack.pop();
            }
        }
    }
}

fn push_unvisited(
    active: &impl Fn(LinkId) -> bool,
    link: LinkId,
    visited: &mut [bool],
    stack: &mut Vec<(LinkId, usize)>,
) {
    if active(link) && !visited[link as usize] {
        visited[link as usize] = true;
        stack.push((link, 0));
    }
}

fn collect_backward(
    graph: &RouteGraph,
    active: &impl Fn(LinkId) -> bool,
    start: LinkId,
    assigned: &mut [bool],
) -> Vec<LinkId> {
    assigned[start as usize] = true;
    let mut stack = vec![start];
    let mut component = Vec::new();
    while let Some(link) = stack.pop() {
        component.push(link);
        for pred in graph.predecessors(link) {
            let from = pred.from;
            if active(from) && !assigned[from as usize] {
                assigned[from as usize] = true;
                stack.push(from);
            }
        }
    }
    component
}
