mod approach_markers;
mod junction_shape;
mod map;
mod setbacks;
mod signal_pills;

pub use approach_markers::{
    ApproachMarkers, MARKER_NONE, MARKER_SIGNAL, MARKER_STOP, MARKER_YIELD, approach_markers,
};
pub use junction_shape::{JunctionShapes, junction_shapes};
pub use map::{AreaRender, NodeRender, RoadRender, area_render, node_render, road_render};
pub use setbacks::road_setbacks;
pub use signal_pills::{
    PILL_AMBER, PILL_GREEN, PILL_RED, SignalPills, signal_pills, signal_states,
};

pub const FILLET_RADIUS_MAX: f64 = 6.0;
pub const ARROW_SPACING: f64 = 40.0;
const JUNCTION_MIN_DEGREE: usize = 3;
