use trapiks_sim_core::challenge::{Center, Challenge, FailReason, RunResult, score};

fn challenge() -> Challenge {
    Challenge {
        id: "test".into(),
        name: "Test".into(),
        blurb: String::new(),
        center: Center::Local { x: 0.0, y: 0.0 },
        radius_m: 500.0,
        seed: 1,
        vehicles_per_hour: 1000.0,
        budget: 100,
        target_improvement: 0.25,
        star_steps: [0.25, 0.5],
        min_throughput_ratio: 0.5,
    }
}

fn run(mean_delay_s: f64, throughput_per_hour: f64) -> RunResult {
    RunResult {
        mean_delay_s,
        throughput_per_hour,
        created: 0,
        arrivals: 0,
        unserved: 0,
        stranded: 0,
    }
}

#[test]
fn pass_gives_one_star() {
    let result = score(&challenge(), &run(100.0, 1000.0), &run(75.0, 1000.0), 100);
    assert!(result.passed);
    assert_eq!(result.stars, 1);
    assert_eq!(result.improvement, 0.25);
    assert_eq!(result.throughput_ratio, 1.0);
    assert!(result.reasons.is_empty());
}

#[test]
fn stars_at_boundaries() {
    let c = challenge();
    let base = run(100.0, 1000.0);
    assert_eq!(score(&c, &base, &run(50.0, 1000.0), 0).stars, 2);
    assert_eq!(score(&c, &base, &run(50.5, 1000.0), 0).stars, 1);
    assert_eq!(score(&c, &base, &run(25.0, 1000.0), 0).stars, 3);
    assert_eq!(score(&c, &base, &run(25.5, 1000.0), 0).stars, 2);
    assert_eq!(score(&c, &base, &run(76.0, 1000.0), 0).stars, 0);
}

#[test]
fn each_fail_reason() {
    let c = challenge();
    let base = run(100.0, 1000.0);
    let slow = score(&c, &base, &run(80.0, 1000.0), 0);
    assert_eq!(slow.reasons, vec![FailReason::NotEnoughImprovement]);
    let starved = score(&c, &base, &run(10.0, 499.0), 0);
    assert_eq!(starved.reasons, vec![FailReason::ThroughputDropped]);
    let pricey = score(&c, &base, &run(10.0, 1000.0), 101);
    assert_eq!(pricey.reasons, vec![FailReason::OverBudget]);
    let all = score(&c, &base, &run(100.0, 0.0), 101);
    assert_eq!(
        all.reasons,
        vec![
            FailReason::NotEnoughImprovement,
            FailReason::ThroughputDropped,
            FailReason::OverBudget
        ]
    );
    assert!(!all.passed);
    assert_eq!(all.stars, 0);
    assert_eq!(pricey.stars, 0);
}

#[test]
fn zero_baseline() {
    let result = score(&challenge(), &run(0.0, 0.0), &run(0.0, 0.0), 0);
    assert_eq!(result.improvement, 0.0);
    assert_eq!(result.throughput_ratio, 1.0);
    assert_eq!(result.reasons, vec![FailReason::NotEnoughImprovement]);
}
