use trapiks_sim_core::demand::DemandTables;
use trapiks_sim_core::edit::EditCommand;
use trapiks_sim_core::fixtures::{GridCity, grid_city};
use trapiks_sim_core::sim::Sim;

fn grid_sim() -> Sim {
    let map = grid_city(&GridCity {
        cols: 5,
        rows: 5,
        spacing: 120.0,
    });
    Sim::new(&map, 3)
}

fn rebuilt(sim: &Sim) -> DemandTables {
    DemandTables::build(sim.network(), sim.route_graph(), sim.link_costs())
}

#[test]
fn delete_updates_tables_one_step_later() {
    let mut sim = grid_sim();
    let before = sim.demand_tables().clone();
    sim.enqueue(EditCommand::DeleteRoad { road: 7 });
    sim.step();
    assert_eq!(sim.demand_tables(), &before);
    sim.step();
    assert_eq!(sim.demand_tables(), &rebuilt(&sim));
    assert_ne!(sim.demand_tables(), &before);
}

#[test]
fn edits_in_one_step_coalesce() {
    let mut sim = grid_sim();
    sim.enqueue(EditCommand::DeleteRoad { road: 7 });
    sim.enqueue(EditCommand::DeleteRoad { road: 12 });
    sim.step();
    sim.step();
    assert_eq!(sim.demand_tables(), &rebuilt(&sim));
}

#[test]
fn undo_before_refresh_restores_tables() {
    let mut sim = grid_sim();
    let before = sim.demand_tables().clone();
    sim.enqueue(EditCommand::DeleteRoad { road: 7 });
    sim.step();
    sim.enqueue(EditCommand::Undo);
    sim.step();
    sim.step();
    assert_eq!(sim.demand_tables(), &before);
    assert_eq!(sim.demand_tables(), &rebuilt(&sim));
}
