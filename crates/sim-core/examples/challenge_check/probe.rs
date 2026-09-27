use std::collections::BTreeMap;

use trapiks_sim_core::challenge::{Challenge, RunResult, score};
use trapiks_sim_core::config::{SimConfig, SimMode};
use trapiks_sim_core::edit::{EditCommand, EditError, Outcome};
use trapiks_sim_core::map::{Control, MapData};
use trapiks_sim_core::network::{TurnKind, road_of};
use trapiks_sim_core::sim::Sim;

use crate::baseline::{print_result, run_with};

const LONG_GREEN_S: u32 = 50;
const SHORT_GREEN_S: u32 = 20;
const EVEN_GREEN_S: u32 = 30;
const FAVOURED_PHASES: usize = 3;

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

    pub fn probes(&self, samples: &BTreeMap<u16, u64>) -> Vec<Probe> {
        let mut probes = Vec::new();
        probes.extend(self.flyover());
        probes.extend(self.retimings());
        probes.extend(self.left_ban(samples));
        probes
    }

    fn flyover(&self) -> Option<Probe> {
        let (sim, _) = self.scratch(&[]);
        let inspection = sim.inspect_node(self.node)?;
        let lanes = |road: u32| {
            sim.inspect_road(road)
                .map_or(0, |r| u32::from(r.lanes_forward + r.lanes_backward))
        };
        let through = inspection
            .flyover_pairs
            .into_iter()
            .max_by_key(|pair| (lanes(pair[0]) + lanes(pair[1]), std::cmp::Reverse(*pair)))?;
        Some(Probe {
            label: format!("flyover through roads {through:?}"),
            commands: vec![EditCommand::BuildFlyover {
                node: self.node,
                through,
            }],
        })
    }

    fn retimings(&self) -> Vec<Probe> {
        let (sim, _) = self.scratch(&[]);
        let prefix = signal_prefix(&sim, self.node);
        let (signalized, _) = self.scratch(&prefix);
        let Some(signal) = signalized.inspect_node(self.node).and_then(|n| n.signal) else {
            return Vec::new();
        };
        green_splits(signal.phase_count as usize)
            .into_iter()
            .map(|greens_s| self.retiming(&prefix, greens_s))
            .collect()
    }

    fn retiming(&self, prefix: &[EditCommand], greens_s: Vec<u32>) -> Probe {
        let mut commands = prefix.to_vec();
        let label = format!(
            "signal greens {greens_s:?}{}",
            if prefix.is_empty() {
                ""
            } else {
                " (new signal)"
            }
        );
        commands.push(EditCommand::SetSignalTiming {
            node: self.node,
            greens_s,
            offset_s: 0,
        });
        Probe { label, commands }
    }

    fn left_ban(&self, samples: &BTreeMap<u16, u64>) -> Option<Probe> {
        let mut sim = self.scratch(&[]).0;
        let junction = sim.network_mut().ensure_junction(self.node).clone();
        let (index, count) = samples
            .iter()
            .filter(|&(&m, _)| {
                junction
                    .movements
                    .get(usize::from(m))
                    .is_some_and(|mv| mv.kind == TurnKind::Left)
            })
            .max_by_key(|&(&m, &count)| (count, std::cmp::Reverse(m)))?;
        let movement = &junction.movements[usize::from(*index)];
        let (from_road, to_road) = (road_of(movement.from_link), road_of(movement.to_link));
        Some(Probe {
            label: format!("ban left {from_road}->{to_road} ({count} samples)"),
            commands: vec![EditCommand::SetTurnAllowed {
                node: self.node,
                from_road,
                to_road,
                allowed: false,
            }],
        })
    }

    pub fn evaluate(&self, probe: &Probe, baseline: &RunResult) {
        println!("  probe {}", probe.label);
        let cost = match self.scratch(&probe.commands).1 {
            Ok(cost) => cost,
            Err(error) => return println!("    rejected: {error:?}"),
        };
        println!(
            "    cost {cost} / budget {} ({})",
            self.challenge.budget,
            if cost <= self.challenge.budget {
                "ok"
            } else {
                "over"
            }
        );
        match run_with(self.map, self.challenge, &probe.commands) {
            Ok(after) => self.report(baseline, &after, cost),
            Err(error) => println!("    run failed: {error:?}"),
        }
    }

    fn report(&self, baseline: &RunResult, after: &RunResult, cost: i64) {
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
    }
}

fn total_cost(sim: &mut Sim) -> Result<i64, EditError> {
    sim.take_results()
        .into_iter()
        .try_fold(0, |sum, result| match result.outcome {
            Outcome::Ok(outcome) => Ok(sum + outcome.cost),
            Outcome::Err(error) => Err(error),
        })
}

fn signal_prefix(sim: &Sim, node: u32) -> Vec<EditCommand> {
    let signalized = sim
        .inspect_node(node)
        .is_some_and(|n| n.control == Control::Signal);
    if signalized {
        return Vec::new();
    }
    vec![EditCommand::SetJunctionControl {
        node,
        control: Control::Signal,
    }]
}

fn green_splits(phases: usize) -> Vec<Vec<u32>> {
    let mut splits = vec![vec![EVEN_GREEN_S; phases]];
    for favoured in 0..phases.min(FAVOURED_PHASES) {
        let split = (0..phases)
            .map(|phase| {
                if phase == favoured {
                    LONG_GREEN_S
                } else {
                    SHORT_GREEN_S
                }
            })
            .collect();
        splits.push(split);
    }
    splits
}
