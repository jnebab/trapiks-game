use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::config::{SimConfig, SimMode};
use crate::edit::{EditCommand, EditError, Outcome};
use crate::map::MapData;
use crate::sim::Sim;
use crate::stats::StatsSnapshot;

use super::Challenge;

pub const WARMUP_TICKS: u64 = 3000;
pub const MEASURE_TICKS: u64 = 6000;
pub const UNSERVED_DELAY: f64 = 300.0;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RunResult {
    pub mean_delay_s: f64,
    pub throughput_per_hour: f64,
    pub created: u64,
    pub arrivals: u64,
    pub unserved: u64,
    pub stranded: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum RunState {
    Running { done_ticks: u64, total_ticks: u64 },
    Finished(RunResult),
}

pub struct ChallengeRun {
    sim: Sim,
    warmup: u64,
    measure: u64,
    finished: Option<RunResult>,
}

impl ChallengeRun {
    pub fn new(
        map: &MapData,
        challenge: &Challenge,
        commands: &[EditCommand],
    ) -> Result<ChallengeRun, EditError> {
        Self::build(map, challenge, commands, (WARMUP_TICKS, MEASURE_TICKS))
    }

    #[cfg(feature = "fixtures")]
    pub fn with_ticks(
        map: &MapData,
        challenge: &Challenge,
        commands: &[EditCommand],
        warmup: u64,
        measure: u64,
    ) -> Result<ChallengeRun, EditError> {
        Self::build(map, challenge, commands, (warmup, measure))
    }

    fn build(
        map: &MapData,
        challenge: &Challenge,
        commands: &[EditCommand],
        (warmup, measure): (u64, u64),
    ) -> Result<ChallengeRun, EditError> {
        let mut sim = Sim::from_config(map, &config(map, challenge));
        for command in commands {
            sim.enqueue(command.clone());
        }
        sim.apply_queued();
        first_error(&mut sim)?;
        Ok(ChallengeRun {
            sim,
            warmup,
            measure,
            finished: None,
        })
    }

    pub fn sim(&self) -> &Sim {
        &self.sim
    }

    pub fn advance(&mut self, max_steps: u32) -> RunState {
        for _ in 0..max_steps {
            if let Some(result) = self.finished {
                return RunState::Finished(result);
            }
            self.step_once();
        }
        self.state()
    }

    fn step_once(&mut self) {
        self.sim.step();
        let tick = self.sim.tick();
        if tick == self.warmup {
            self.sim.reset_stats();
        }
        if tick == self.warmup + self.measure {
            self.finished = Some(result_of(&self.sim.stats()));
        }
    }

    fn state(&self) -> RunState {
        match self.finished {
            Some(result) => RunState::Finished(result),
            None => RunState::Running {
                done_ticks: self.sim.tick(),
                total_ticks: self.warmup + self.measure,
            },
        }
    }
}

fn config(map: &MapData, challenge: &Challenge) -> SimConfig {
    let (center_x, center_y) = challenge.center_xy(map);
    SimConfig {
        seed: challenge.seed,
        mode: SimMode::Region {
            center_x,
            center_y,
            radius: challenge.radius_m,
        },
        vehicles_per_hour: challenge.vehicles_per_hour,
        budget: Some(challenge.budget),
    }
}

fn first_error(sim: &mut Sim) -> Result<(), EditError> {
    let failure = sim
        .take_results()
        .into_iter()
        .find_map(|r| match r.outcome {
            Outcome::Err(error) => Some(error),
            Outcome::Ok(_) => None,
        });
    failure.map_or(Ok(()), Err)
}

fn result_of(stats: &StatsSnapshot) -> RunResult {
    let penalized = (stats.unserved + stats.stranded) as f64 * UNSERVED_DELAY;
    RunResult {
        mean_delay_s: (stats.accrued_delay_s + penalized) / stats.created.max(1) as f64,
        throughput_per_hour: stats.throughput_per_hour,
        created: stats.created,
        arrivals: stats.arrivals,
        unserved: stats.unserved,
        stranded: stats.stranded,
    }
}
