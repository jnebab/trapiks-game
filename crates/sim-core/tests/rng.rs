use trapiks_sim_core::rng::Pcg32;

#[test]
fn matches_the_reference_vector() {
    let mut rng = Pcg32::new(42, 54);
    let drawn: Vec<u32> = (0..6).map(|_| rng.next_u32()).collect();
    assert_eq!(
        drawn,
        vec![
            0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e
        ]
    );
}

#[test]
fn below_stays_in_range() {
    let mut rng = Pcg32::new(1, 2);
    assert!((0..10_000).all(|_| rng.below(10) < 10));
}

#[test]
fn next_f64_is_in_unit_interval() {
    let mut rng = Pcg32::new(3, 4);
    assert!((0..10_000).all(|_| (0.0..1.0).contains(&rng.next_f64())));
}
