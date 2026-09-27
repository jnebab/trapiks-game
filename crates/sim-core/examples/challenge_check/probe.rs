use trapiks_sim_core::challenge::{Challenge, RunResult, Score, score};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{EditCommand, EditError, Outcome};
use trapiks_sim_core::map::MapData;
use trapiks_sim_core::sim::Sim;

use crate::baseline::{Samples, print_result, run_with};
use crate::variants::{extra_lane, flyovers, green_splits, left_bans, roundabouts, signal_prefix};

pub struct Probe {
    pub label: String,
    pub commands: Vec<EditCommand>,
}

pub struct Context<'a> {
    pub map: &'a MapData,
    pub challenge: &'a Challenge,
    pub node: u32,
}

impl Context<'_> {
    fn scratch(&self, commands: &[EditCommand]) -> (Sim, Result<i64, EditError>) {
        let (center_x, center_y) = self.challenge.center_xy(self.map);
        let config = SimConfig {
            seed: self.challenge.seed,
            mode: SimMode::Region {
                center_x,
                center_y,
                radius: self.challenge.radius_m,
            },
            vehicles_per_hour: self.challenge.vehicles_per_hour,
            budget: Some(self.challenge.budget),
        };
        let mut sim = Sim::from_config(self.map, &config);
        for command in commands {
            sim.enqueue(command.clone());
        }
        sim.apply_queued();
        let cost = total_cost(&mut sim);
        (sim, cost)
    }

    pub fn probes(&self, samples: &Samples) -> Vec<Probe> {
        let mut sim = self.scratch(&[]).0;
        let junction = sim.network_mut().ensure_junction(self.node).clone();
        let mut probes = flyovers(&sim, self.node);
        probes.extend(self.retimings(&sim));
        probes.extend(left_bans(&junction, self.node, samples));
        probes.extend(extra_lane(&sim, samples));
        probes.extend(roundabouts(&sim, self.node));
        probes
    }

    fn retimings(&self, sim: &Sim) -> Vec<Probe> {
        let prefix = signal_prefix(sim, self.node);
        let (signalized, _) = self.scratch(&prefix);
        let Some(signal) = signalized.inspect_node(self.node).and_then(|n| n.signal) else {
            return Vec::new();
        };
        green_splits(signal.phase_count as usize)
            .into_iter()
            .map(|greens_s| retiming(self.node, &prefix, greens_s))
            .collect()
    }

    pub fn evaluate(&self, probe: &Probe, baseline: &RunResult) -> Option<Score> {
        println!("  probe {}", probe.label);
        let cost = match self.scratch(&probe.commands).1 {
            Ok(cost) => cost,
            Err(error) => {
                println!("    rejected: {error:?}");
                return None;
            }
        };
        println!("    cost {cost} / budget {}", self.challenge.budget);
        match run_with(self.map, self.challenge, &probe.commands) {
            Ok(after) => Some(self.report(baseline, &after, cost)),
            Err(error) => {
                println!("    run failed: {error:?}");
                None
            }
        }
    }

    fn report(&self, baseline: &RunResult, after: &RunResult, cost: i64) -> Score {
        print_result("  after", after);
        let scored = score(self.challenge, baseline, after, cost);
        println!(
            "    improvement {:.1} % throughput_ratio {:.3} stars {} passed {} reasons {:?}",
            scored.improvement * 100.0,
            scored.throughput_ratio,
            scored.stars,
            scored.passed,
            scored.reasons
        );
        scored
    }
}

fn retiming(node: u32, prefix: &[EditCommand], greens_s: Vec<u32>) -> Probe {
    let mut commands = prefix.to_vec();
    let suffix = if prefix.is_empty() {
        ""
    } else {
        " (new signal)"
    };
    let label = format!("signal greens {greens_s:?}{suffix}");
    commands.push(EditCommand::SetSignalTiming {
        node,
        greens_s,
        offset_s: 0,
    });
    Probe { label, commands }
}

fn total_cost(sim: &mut Sim) -> Result<i64, EditError> {
    sim.take_results()
        .into_iter()
        .try_fold(0, |sum, result| match result.outcome {
            Outcome::Ok(outcome) => Ok(sum + outcome.cost),
            Outcome::Err(error) => Err(error),
        })
}
