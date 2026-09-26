use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Challenge, RunResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FailReason {
    NotEnoughImprovement,
    ThroughputDropped,
    OverBudget,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Score {
    pub improvement: f64,
    pub throughput_ratio: f64,
    pub cost: i64,
    pub passed: bool,
    pub stars: u8,
    pub reasons: Vec<FailReason>,
}

pub fn score(challenge: &Challenge, baseline: &RunResult, after: &RunResult, cost: i64) -> Score {
    let improvement = improvement(baseline, after);
    let throughput_ratio = throughput_ratio(baseline, after);
    let checks = [
        (
            improvement >= challenge.target_improvement,
            FailReason::NotEnoughImprovement,
        ),
        (
            throughput_ratio >= challenge.min_throughput_ratio,
            FailReason::ThroughputDropped,
        ),
        (cost <= challenge.budget, FailReason::OverBudget),
    ];
    let reasons: Vec<FailReason> = checks
        .into_iter()
        .filter(|(ok, _)| !ok)
        .map(|(_, reason)| reason)
        .collect();
    let passed = reasons.is_empty();
    Score {
        improvement,
        throughput_ratio,
        cost,
        passed,
        stars: stars(challenge, improvement, passed),
        reasons,
    }
}

fn improvement(baseline: &RunResult, after: &RunResult) -> f64 {
    if baseline.mean_delay_s == 0.0 {
        return 0.0;
    }
    1.0 - after.mean_delay_s / baseline.mean_delay_s
}

fn throughput_ratio(baseline: &RunResult, after: &RunResult) -> f64 {
    if baseline.throughput_per_hour == 0.0 {
        return 1.0;
    }
    after.throughput_per_hour / baseline.throughput_per_hour
}

fn stars(challenge: &Challenge, improvement: f64, passed: bool) -> u8 {
    if !passed {
        return 0;
    }
    let target = challenge.target_improvement;
    let bonus = challenge
        .star_steps
        .iter()
        .filter(|&&step| improvement >= target + step)
        .count();
    1 + bonus as u8
}
