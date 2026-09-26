use trapiks_sim_core::fixtures::{
    FOUR_WAY_HASH_5000, GRID_CITY_HASH_1000, four_way_5000, grid_city_demand_1000,
};

#[test]
fn four_way_hash_matches_constant() {
    let hash = four_way_5000();
    assert_eq!(hash, FOUR_WAY_HASH_5000, "{hash:#018x}");
}

#[test]
fn grid_city_hash_matches_constant() {
    let hash = grid_city_demand_1000();
    assert_eq!(hash, GRID_CITY_HASH_1000, "{hash:#018x}");
}
