use super::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
    pub p3: Vec2,
}

impl CubicBezier {
    pub fn point(&self, t: f64) -> Vec2 {
        let u = 1.0 - t;
        self.p0 * (u * u * u)
            + self.p1 * (3.0 * u * u * t)
            + self.p2 * (3.0 * u * t * t)
            + self.p3 * (t * t * t)
    }

    pub fn sample(&self, segments: usize) -> Vec<Vec2> {
        let segments = segments.max(1);
        (0..=segments)
            .map(|i| self.point(i as f64 / segments as f64))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadraticBezier {
    pub p0: Vec2,
    pub p1: Vec2,
    pub p2: Vec2,
}

impl QuadraticBezier {
    pub fn point(&self, t: f64) -> Vec2 {
        let u = 1.0 - t;
        self.p0 * (u * u) + self.p1 * (2.0 * u * t) + self.p2 * (t * t)
    }
}
