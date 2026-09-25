use super::Vec2;

pub fn cumulative_lengths(points: &[Vec2]) -> Vec<f64> {
    let mut total = 0.0;
    let mut result = Vec::with_capacity(points.len());
    result.push(0.0);
    for pair in points.windows(2) {
        total += pair[0].distance(pair[1]);
        result.push(total);
    }
    result.truncate(points.len());
    result
}

pub fn pose_at(points: &[Vec2], cumulative: &[f64], s: f64) -> (Vec2, Vec2) {
    let count = points.len().min(cumulative.len());
    if count < 2 {
        return (points.first().copied().unwrap_or_default(), Vec2::default());
    }
    let total = cumulative[count - 1];
    let s = s.clamp(0.0, total);
    let segment = segment_index(&cumulative[..count], s);
    let (a, b) = (points[segment], points[segment + 1]);
    let span = cumulative[segment + 1] - cumulative[segment];
    let t = if span > 0.0 {
        (s - cumulative[segment]) / span
    } else {
        0.0
    };
    (Vec2::lerp(a, b, t), (b - a).normalized())
}

fn segment_index(cumulative: &[f64], s: f64) -> usize {
    let last_segment = cumulative.len() - 2;
    let upper = cumulative.partition_point(|&c| c <= s);
    upper.saturating_sub(1).min(last_segment)
}
