use trapiks_sim_core::challenge::{RunResult, Score};

use crate::baseline::Samples;
use crate::probe::{Context, Probe};

const COMBINE_TOP: usize = 4;

struct Scored {
    probe: Probe,
    score: Score,
}

pub fn run_probes(context: &Context, samples: &Samples, baseline: &RunResult) {
    let mut singles: Vec<Scored> = context
        .probes(samples)
        .into_iter()
        .filter_map(|probe| {
            let score = context.evaluate(&probe, baseline)?;
            Some(Scored { probe, score })
        })
        .collect();
    singles.sort_by(|a, b| rank(&b.score).total_cmp(&rank(&a.score)));
    print_best("best probe", singles.first());
    let pairs = combine(context, &singles, baseline);
    print_best("best combination", best_of(pairs).as_ref());
}

fn rank(score: &Score) -> f64 {
    f64::from(score.stars) * 10.0 + score.improvement
}

fn combine(context: &Context, singles: &[Scored], baseline: &RunResult) -> Vec<Scored> {
    let top: Vec<&Scored> = singles
        .iter()
        .filter(|s| s.score.improvement > 0.0)
        .take(COMBINE_TOP)
        .collect();
    let mut pairs = Vec::new();
    for (i, first) in top.iter().enumerate() {
        for second in &top[i + 1..] {
            let probe = joined(&first.probe, &second.probe);
            if let Some(score) = context.evaluate(&probe, baseline) {
                pairs.push(Scored { probe, score });
            }
        }
    }
    pairs
}

fn joined(first: &Probe, second: &Probe) -> Probe {
    let mut commands = first.commands.clone();
    commands.extend(second.commands.iter().cloned());
    Probe {
        label: format!("{} + {}", first.label, second.label),
        commands,
    }
}

fn best_of(candidates: Vec<Scored>) -> Option<Scored> {
    candidates.into_iter().reduce(|best, next| {
        if rank(&next.score) > rank(&best.score) {
            next
        } else {
            best
        }
    })
}

fn print_best(title: &str, best: Option<&Scored>) {
    let Some(best) = best else {
        println!("  {title}: none");
        return;
    };
    println!(
        "  {title}: {}: improvement {:.1} % throughput_ratio {:.3} stars {} cost {}",
        best.probe.label,
        best.score.improvement * 100.0,
        best.score.throughput_ratio,
        best.score.stars,
        best.score.cost
    );
}
