use crate::consts::BEZIER_SEGMENTS;
use crate::geom::{Bounds, bounds, intersect};

use super::{Conflict, Movement};

pub fn find(movements: &[Movement]) -> Vec<Vec<Conflict>> {
    let boxes: Vec<Bounds> = movements.iter().map(|m| bounds(m.path)).collect();
    let mut lists = vec![Vec::new(); movements.len()];
    for (i, a) in movements.iter().enumerate() {
        for (j, b) in movements.iter().enumerate().skip(i + 1) {
            if let Some((s_i, s_j, merge)) = pair_conflict(a, b, boxes[i], boxes[j]) {
                lists[i].push(conflict(j, s_i, s_j, merge));
                lists[j].push(conflict(i, s_j, s_i, merge));
            }
        }
    }
    for list in &mut lists {
        list.sort_by(|x, y| x.s_self.total_cmp(&y.s_self).then(x.other.cmp(&y.other)));
    }
    lists
}

fn conflict(other: usize, s_self: f64, s_other: f64, merge: bool) -> Conflict {
    Conflict {
        other: u16::try_from(other).unwrap_or(u16::MAX),
        s_self,
        s_other,
        merge,
    }
}

fn pair_conflict(
    a: &Movement,
    b: &Movement,
    a_box: Bounds,
    b_box: Bounds,
) -> Option<(f64, f64, bool)> {
    if a.from_link == b.from_link {
        return None;
    }
    if a.to_link == b.to_link {
        return Some((a.length, b.length, true));
    }
    if !overlap(a_box, b_box) {
        return None;
    }
    first_crossing(a, b).map(|(s_a, s_b)| (s_a, s_b, false))
}

fn first_crossing(a: &Movement, b: &Movement) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64)> = None;
    for i in 0..BEZIER_SEGMENTS {
        for j in 0..BEZIER_SEGMENTS {
            let Some((t, u)) = intersect(a.path[i], a.path[i + 1], b.path[j], b.path[j + 1]) else {
                continue;
            };
            let found = (along(a, i, t), along(b, j, u));
            if best.is_none_or(|current| earlier(found, current)) {
                best = Some(found);
            }
        }
    }
    best
}

fn earlier(candidate: (f64, f64), current: (f64, f64)) -> bool {
    candidate
        .0
        .total_cmp(&current.0)
        .then(candidate.1.total_cmp(&current.1))
        .is_lt()
}

fn along(m: &Movement, segment: usize, t: f64) -> f64 {
    let start = m.cumulative[segment];
    start + t * (m.cumulative[segment + 1] - start)
}

fn overlap((a_min, a_max): Bounds, (b_min, b_max): Bounds) -> bool {
    a_min.x <= b_max.x && b_min.x <= a_max.x && a_min.y <= b_max.y && b_min.y <= a_max.y
}
