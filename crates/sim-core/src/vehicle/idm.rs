use crate::consts::{IDM_COMFORT_DECEL, IDM_MIN_GAP, IDM_TIME_HEADWAY};

const MIN_GAP_CLAMP: f64 = 0.1;

pub fn acceleration(v: f64, v0: f64, gap: f64, dv: f64, a: f64) -> f64 {
    let r = v / v0;
    let free = r * r * r * r;
    a * (1.0 - free - interaction(v, gap, dv, a))
}

pub fn interaction(v: f64, gap: f64, dv: f64, a: f64) -> f64 {
    if gap == f64::INFINITY {
        return 0.0;
    }
    let brake = 2.0 * libm::sqrt(a * IDM_COMFORT_DECEL);
    let desired = IDM_MIN_GAP + (v * IDM_TIME_HEADWAY + v * dv / brake).max(0.0);
    let ratio = desired / gap.max(MIN_GAP_CLAMP);
    ratio * ratio
}
