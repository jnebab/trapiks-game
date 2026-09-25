use trapiks_sim_core::{MAP_FORMAT_VERSION, engine_info};

#[test]
fn engine_info_reports_name_and_map_format_version() {
    let info = engine_info();
    assert_eq!(info.name, "Trapiks");
    assert_eq!(info.map_format_version, MAP_FORMAT_VERSION);
    assert_eq!(info.map_format_version, 1);
}
