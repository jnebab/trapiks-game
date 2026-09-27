use std::collections::BTreeMap;
use std::time::Instant;

use trapiks_sim_core::challenge::{Challenge, ChallengeRun, RunResult, RunState};
use trapiks_sim_core::edit::EditError;
use trapiks_sim_core::map::MapData;
use trapiks_sim_core::network::LinkId;
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::Place;

const SAMPLE_TICKS: u32 = 10;

pub struct Baseline {
    pub result: RunResult,
    pub wall_s: f64,
    pub samples: Samples,
}

#[derive(Default)]
pub struct Samples {
    pub movements: BTreeMap<u16, u64>,
    pub approaches: BTreeMap<LinkId, u64>,
}

pub fn run_baseline(
    map: &MapData,
    challenge: &Challenge,
    node: u32,
) -> Result<Baseline, EditError> {
    let started = Instant::now();
    let mut run = ChallengeRun::new(map, challenge, &[])?;
    let mut samples = Samples::default();
    loop {
        if let RunState::Finished(result) = run.advance(SAMPLE_TICKS) {
            return Ok(Baseline {
                result,
                wall_s: started.elapsed().as_secs_f64(),
                samples,
            });
        }
        sample(run.sim(), node, &mut samples);
    }
}

fn sample(sim: &Sim, node: u32, samples: &mut Samples) {
    let vehicles = sim.vehicles();
    for slot in vehicles.live_slots() {
        match vehicles.place[slot as usize] {
            Place::Movement {
                node: at, movement, ..
            } if at == node => *samples.movements.entry(movement).or_default() += 1,
            Place::Link { link, .. } if sim.network().link_to(link) == node => {
                *samples.approaches.entry(link).or_default() += 1;
            }
            _ => {}
        }
    }
}

pub fn run_with(
    map: &MapData,
    challenge: &Challenge,
    commands: &[trapiks_sim_core::edit::EditCommand],
) -> Result<RunResult, EditError> {
    let mut run = ChallengeRun::new(map, challenge, commands)?;
    loop {
        if let RunState::Finished(result) = run.advance(u32::MAX) {
            return Ok(result);
        }
    }
}

pub fn print_result(label: &str, result: &RunResult) {
    println!(
        "  {label}: mean_delay_s={:.1} throughput_per_hour={:.0} created={} arrivals={} unserved={} stranded={}",
        result.mean_delay_s,
        result.throughput_per_hour,
        result.created,
        result.arrivals,
        result.unserved,
        result.stranded
    );
}
