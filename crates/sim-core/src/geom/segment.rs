use super::Vec2;

const PARALLEL_EPSILON: f64 = 1e-12;

pub fn intersect(a0: Vec2, a1: Vec2, b0: Vec2, b1: Vec2) -> Option<(f64, f64)> {
    let r = a1 - a0;
    let q = b1 - b0;
    let denominator = r.cross(q);
    if denominator.abs() < PARALLEL_EPSILON {
        return None;
    }
    let offset = b0 - a0;
    let t = offset.cross(q) / denominator;
    let u = offset.cross(r) / denominator;
    let inside = (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u);
    inside.then_some((t, u))
}
