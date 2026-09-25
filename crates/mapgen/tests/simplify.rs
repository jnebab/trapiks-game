use trapiks_mapgen::simplify::{self, TOLERANCE_M, simplify_ring};

#[test]
fn douglas_peucker() {
    let collinear = [(0.0, 0.0), (10.0, 0.0), (20.0, 0.0), (30.0, 0.0)];
    assert_eq!(
        simplify::douglas_peucker(&collinear, TOLERANCE_M),
        vec![(0.0, 0.0), (30.0, 0.0)]
    );
    let zig_zag: Vec<(f64, f64)> = (0..10)
        .map(|i| (f64::from(i) * 10.0, f64::from(i % 2)))
        .collect();
    assert_eq!(simplify::douglas_peucker(&zig_zag, TOLERANCE_M).len(), 2);
    let spike = [
        (0.0, 0.0),
        (10.0, 0.0),
        (15.0, 5.0),
        (20.0, 0.0),
        (30.0, 0.0),
    ];
    assert!(simplify::douglas_peucker(&spike, TOLERANCE_M).contains(&(15.0, 5.0)));
}

#[test]
fn ring_simplification() {
    let square = [
        (0.0, 0.0),
        (50.0, 0.0),
        (100.0, 0.0),
        (100.0, 100.0),
        (0.0, 100.0),
    ];
    assert_eq!(
        simplify_ring(&square, TOLERANCE_M).map(|r| r.len()),
        Some(4)
    );
    let tiny = [(0.0, 0.0), (5.0, 0.0), (5.0, 5.0), (0.0, 5.0)];
    assert_eq!(simplify_ring(&tiny, TOLERANCE_M), None);
    assert_eq!(simplify_ring(&[(1.0, 1.0); 3], TOLERANCE_M), None);
}
