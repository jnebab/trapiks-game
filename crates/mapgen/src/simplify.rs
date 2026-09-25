pub const TOLERANCE_M: f64 = 1.5;
pub const MIN_RING_AREA_M2: f64 = 100.0;

type Point = (f64, f64);

pub fn douglas_peucker(points: &[Point], tolerance: f64) -> Vec<Point> {
    if points.len() <= 2 {
        return points.to_vec();
    }
    let mut keep = vec![false; points.len()];
    keep[0] = true;
    keep[points.len() - 1] = true;
    let mut stack = vec![(0, points.len() - 1)];
    while let Some((start, end)) = stack.pop() {
        let Some((index, distance)) = farthest_from_segment(points, start, end) else {
            continue;
        };
        if distance > tolerance {
            keep[index] = true;
            stack.push((start, index));
            stack.push((index, end));
        }
    }
    points
        .iter()
        .zip(keep)
        .filter_map(|(point, kept)| kept.then_some(*point))
        .collect()
}

pub fn simplify_ring(points: &[Point], tolerance: f64) -> Option<Vec<Point>> {
    let first = *points.first()?;
    let (far, distance) = farthest_from_point(points, first);
    if distance == 0.0 {
        return None;
    }
    let half_one = douglas_peucker(&points[..=far], tolerance);
    let mut second: Vec<Point> = points[far..].to_vec();
    second.push(first);
    let half_two = douglas_peucker(&second, tolerance);
    let mut ring = half_one;
    ring.extend_from_slice(&half_two[1..half_two.len() - 1]);
    if ring.len() < 3 || ring_area(&ring) < MIN_RING_AREA_M2 {
        return None;
    }
    Some(ring)
}

pub fn ring_area(points: &[Point]) -> f64 {
    let next = points.iter().cycle().skip(1);
    let twice: f64 = points
        .iter()
        .zip(next)
        .map(|(a, b)| a.0 * b.1 - b.0 * a.1)
        .sum();
    twice.abs() / 2.0
}

fn farthest_from_point(points: &[Point], origin: Point) -> (usize, f64) {
    let mut best = (0, 0.0);
    for (index, point) in points.iter().enumerate() {
        let distance = (point.0 - origin.0).hypot(point.1 - origin.1);
        if distance > best.1 {
            best = (index, distance);
        }
    }
    best
}

fn farthest_from_segment(points: &[Point], start: usize, end: usize) -> Option<(usize, f64)> {
    let (a, b) = (points[start], points[end]);
    (start + 1..end)
        .map(|index| (index, segment_distance(points[index], a, b)))
        .fold(None, |best, candidate| match best {
            Some(best) if best.1 >= candidate.1 => Some(best),
            _ => Some(candidate),
        })
}

fn segment_distance(point: Point, a: Point, b: Point) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length_squared = dx * dx + dy * dy;
    if length_squared == 0.0 {
        return (point.0 - a.0).hypot(point.1 - a.1);
    }
    let t = (((point.0 - a.0) * dx + (point.1 - a.1) * dy) / length_squared).clamp(0.0, 1.0);
    (point.0 - a.0 - t * dx).hypot(point.1 - a.1 - t * dy)
}
