mod support;

use support::Must;
use trapiks_sim_core::challenge::{Center, Challenge};
use trapiks_sim_core::fixtures::MapBuilder;
use trapiks_sim_core::geo::Projection;
use trapiks_sim_core::map::GeoOrigin;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../web/public/challenges/");

fn load(name: &str) -> Vec<Challenge> {
    let text = std::fs::read_to_string(format!("{DIR}{name}")).must("challenge file");
    serde_json::from_str(&text).must("valid json")
}

#[test]
fn files_parse_and_validate() {
    for name in ["synthetic.json", "metro-manila.json"] {
        let challenges = load(name);
        assert!(!challenges.is_empty());
        for challenge in &challenges {
            assert_eq!(challenge.validate(), Ok(()), "{}", challenge.id);
        }
    }
    assert_eq!(load("synthetic.json").len(), 3);
    assert_eq!(load("metro-manila.json").len(), 7);
}

#[test]
fn center_json_is_externally_tagged() {
    let json = serde_json::to_value(Center::Local {
        x: 1500.0,
        y: 1500.0,
    })
    .must("json");
    assert_eq!(
        json,
        serde_json::json!({ "Local": { "x": 1500.0, "y": 1500.0 } })
    );
}

#[test]
fn edsa_ortigas_projects_from_map_origin() {
    let map = MapBuilder::new().build();
    assert_eq!(
        map.origin,
        GeoOrigin {
            lat: 14.5995,
            lon: 120.9842
        }
    );
    let challenges = load("metro-manila.json");
    let edsa = challenges
        .iter()
        .find(|c| c.id == "edsa-ortigas")
        .must("edsa-ortigas");
    let (x, y) = edsa.center_xy(&map);
    let radius = 6_371_008.8_f64;
    let expected_x =
        radius * (121.0567_f64 - 120.9842).to_radians() * 14.5995_f64.to_radians().cos();
    let expected_y = -radius * (14.5869_f64 - 14.5995).to_radians();
    assert!((x - expected_x).abs() < 1.0, "{x} vs {expected_x}");
    assert!((y - expected_y).abs() < 1.0, "{y} vs {expected_y}");
    let projection = Projection::from_origin(map.origin.clone());
    assert_eq!(projection.project(14.5869, 121.0567), (x, y));
}
