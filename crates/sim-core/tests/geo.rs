use trapiks_sim_core::geo::Projection;
use trapiks_sim_core::map::GeoOrigin;

#[test]
fn unproject_inverts_project() {
    let projection = Projection::from_origin(GeoOrigin {
        lat: 14.6,
        lon: 121.0,
    });
    let (x, y) = projection.project(14.5869, 121.0567);
    let (lat, lon) = projection.unproject(x, y);
    assert!((lat - 14.5869).abs() < 1e-9);
    assert!((lon - 121.0567).abs() < 1e-9);
}
