mod support;

use support::apply_now;
use trapiks_sim_core::edit::{EditCommand, EditError, Outcome};
use trapiks_sim_core::fixtures::four_way;
use trapiks_sim_core::sim::Sim;

fn cost_of(outcome: &Outcome) -> Option<i64> {
    match outcome {
        Outcome::Ok(outcome) => Some(outcome.cost),
        Outcome::Err(_) => None,
    }
}

fn second_delete_is_refused(sim: &mut Sim) {
    let command = EditCommand::DeleteRoad { road: 1 };
    assert_eq!(sim.quote(&command), Err(EditError::InsufficientBudget));
    let outcome = apply_now(sim, command);
    assert_eq!(outcome, Outcome::Err(EditError::InsufficientBudget));
}

#[test]
fn budget() {
    let mut sim = Sim::new(&four_way(2, 200.0), 1);
    sim.set_budget_limit(Some(150));
    let first = apply_now(&mut sim, EditCommand::DeleteRoad { road: 0 });
    assert_eq!(cost_of(&first), Some(100));
    assert_eq!(sim.budget().spent, 100);
    second_delete_is_refused(&mut sim);
    assert_eq!(sim.quote(&EditCommand::Undo), Ok(-100));
    let undo = apply_now(&mut sim, EditCommand::Undo);
    assert_eq!(cost_of(&undo), Some(-100));
    assert_eq!(sim.budget().spent, 0);
}
