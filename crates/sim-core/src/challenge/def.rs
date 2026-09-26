use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::edit::MAX_VPH;
use crate::geo::Projection;
use crate::map::MapData;

const RADIUS_RANGE_M: (f64, f64) = (300.0, 4_000.0);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Center {
    LatLon { lat: f64, lon: f64 },
    Local { x: f64, y: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub blurb: String,
    pub center: Center,
    pub radius_m: f64,
    pub seed: u64,
    pub vehicles_per_hour: f64,
    pub budget: i64,
    pub target_improvement: f64,
    pub star_steps: [f64; 2],
    pub min_throughput_ratio: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ChallengeError {
    InvalidRadius,
    InvalidDemand,
    InvalidBudget,
    InvalidTarget,
    InvalidStarSteps,
    InvalidThroughputRatio,
}

impl Challenge {
    pub fn center_xy(&self, map: &MapData) -> (f64, f64) {
        match self.center {
            Center::LatLon { lat, lon } => {
                Projection::from_origin(map.origin.clone()).project(lat, lon)
            }
            Center::Local { x, y } => (x, y),
        }
    }

    pub fn validate(&self) -> Result<(), ChallengeError> {
        let (min_radius, max_radius) = RADIUS_RANGE_M;
        require(
            (min_radius..=max_radius).contains(&self.radius_m),
            ChallengeError::InvalidRadius,
        )?;
        require(
            (0.0..=MAX_VPH).contains(&self.vehicles_per_hour),
            ChallengeError::InvalidDemand,
        )?;
        require(self.budget >= 0, ChallengeError::InvalidBudget)?;
        require(
            self.target_improvement > 0.0 && self.target_improvement < 1.0,
            ChallengeError::InvalidTarget,
        )?;
        require(
            self.star_steps[0] < self.star_steps[1],
            ChallengeError::InvalidStarSteps,
        )?;
        require(
            self.min_throughput_ratio > 0.0 && self.min_throughput_ratio <= 1.0,
            ChallengeError::InvalidThroughputRatio,
        )
    }
}

fn require(condition: bool, error: ChallengeError) -> Result<(), ChallengeError> {
    if condition { Ok(()) } else { Err(error) }
}
