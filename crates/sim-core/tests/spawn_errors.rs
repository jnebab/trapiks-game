use trapiks_sim_core::fixtures::{corridor, one_way_pair};
use trapiks_sim_core::sim::Sim;
use trapiks_sim_core::vehicle::{Place, SpawnError};

#[test]
fn empty_route() {
    let mut sim = Sim::new(&corridor(), 0);
    assert_eq!(sim.spawn(&[]), Err(SpawnError::EmptyRoute));
}

#[test]
fn too_long() {
    let mut sim = Sim::new(&corridor(), 0);
    let route = vec![0; 65_536];
    assert_eq!(sim.spawn(&route), Err(SpawnError::TooLong));
}

#[test]
fn inactive_link() {
    let mut sim = Sim::new(&one_way_pair(), 0);
    assert_eq!(sim.spawn(&[7]), Err(SpawnError::InactiveLink(7)));
    assert_eq!(
        sim.spawn(&[1_000_000]),
        Err(SpawnError::InactiveLink(1_000_000))
    );
}

#[test]
fn disconnected() {
    let mut sim = Sim::new(&corridor(), 0);
    assert_eq!(sim.spawn(&[0, 2, 0]), Err(SpawnError::Disconnected(1)));
}

#[test]
fn full() {
    let mut sim = Sim::with_capacity(&corridor(), 0, 1);
    assert!(sim.spawn(&[0]).is_ok());
    assert_eq!(sim.spawn(&[2]), Err(SpawnError::Full));
}

#[test]
fn blocked() {
    let mut sim = Sim::new(&corridor(), 0);
    assert!(sim.spawn(&[0]).is_ok());
    assert!(sim.spawn(&[0]).is_ok());
    assert_eq!(sim.spawn(&[0]), Err(SpawnError::Blocked));
}

#[test]
fn blocked_by_vehicle_that_just_entered() {
    let mut sim = Sim::new(&corridor(), 0);
    assert!(sim.spawn(&[0, 2, 4]).is_ok());
    let mut steps = 0;
    while !matches!(sim.vehicles().place[0], Place::Link { link: 2, .. }) {
        sim.step();
        steps += 1;
        assert!(steps < 10_000);
    }
    assert!(sim.spawn(&[2, 4]).is_ok());
    assert_eq!(sim.spawn(&[2, 4]), Err(SpawnError::Blocked));
}
