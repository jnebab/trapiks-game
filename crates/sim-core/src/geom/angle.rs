use std::f64::consts::PI;

use super::Vec2;

pub fn signed_turn(from: Vec2, to: Vec2) -> f64 {
    libm::atan2(from.cross(to), from.dot(to))
}

pub fn axis_angle(d: Vec2) -> f64 {
    let angle = libm::atan2(d.y, d.x).rem_euclid(PI);
    if angle >= PI { 0.0 } else { angle }
}
