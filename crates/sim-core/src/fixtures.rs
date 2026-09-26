mod builder;
mod corridor;
mod grid_city;
mod junctions;

pub use builder::{MapBuilder, RoadSpec};
pub use corridor::corridor;
pub use grid_city::{GridCity, grid_city};
pub use junctions::{dead_end, dual_carriageway_cross, four_way, one_way_pair, t_junction};
mod scenarios;
pub use scenarios::{
    FOUR_WAY_HASH_5000, GRID_CITY_HASH_1000, four_way_5000, grid_city_demand_1000,
};
