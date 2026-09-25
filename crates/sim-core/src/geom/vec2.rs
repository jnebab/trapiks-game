use std::ops::{Add, Mul, Neg, Sub};

const MIN_LENGTH: f64 = 1e-12;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn dot(self, other: Vec2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn cross(self, other: Vec2) -> f64 {
        self.x * other.y - self.y * other.x
    }

    pub fn length(self) -> f64 {
        libm::sqrt(self.dot(self))
    }

    pub fn distance(self, other: Vec2) -> f64 {
        (self - other).length()
    }

    pub fn normalized(self) -> Vec2 {
        let length = self.length();
        if length < MIN_LENGTH {
            return Vec2::default();
        }
        Vec2::new(self.x / length, self.y / length)
    }

    pub fn lerp(a: Vec2, b: Vec2, t: f64) -> Vec2 {
        a + (b - a) * t
    }

    pub fn perp_right(self) -> Vec2 {
        Vec2::new(-self.y, self.x)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, other: Vec2) -> Vec2 {
        Vec2::new(self.x - other.x, self.y - other.y)
    }
}

impl Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2::new(-self.x, -self.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}
