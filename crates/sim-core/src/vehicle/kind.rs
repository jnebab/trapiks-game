use crate::consts::IDM_MAX_ACCEL;
use crate::rng::Pcg32;

const CAR_SHARE: u32 = 80;
const JEEPNEY_SHARE_END: u32 = 95;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum VehicleKind {
    #[default]
    Car,
    Jeepney,
    Bus,
}

impl VehicleKind {
    pub fn draw(rng: &mut Pcg32) -> VehicleKind {
        match rng.below(100) {
            roll if roll < CAR_SHARE => VehicleKind::Car,
            roll if roll < JEEPNEY_SHARE_END => VehicleKind::Jeepney,
            _ => VehicleKind::Bus,
        }
    }

    pub fn length(self) -> f64 {
        match self {
            VehicleKind::Car => 4.4,
            VehicleKind::Jeepney => 6.5,
            VehicleKind::Bus => 12.0,
        }
    }

    pub fn accel(self) -> f64 {
        match self {
            VehicleKind::Car => IDM_MAX_ACCEL,
            VehicleKind::Jeepney => 1.0,
            VehicleKind::Bus => 0.8,
        }
    }

    pub fn speed_factor(self) -> f64 {
        match self {
            VehicleKind::Car => 1.0,
            VehicleKind::Jeepney => 0.9,
            VehicleKind::Bus => 0.85,
        }
    }

    pub fn code(self) -> u8 {
        self as u8
    }
}
