use trapiks_sim_core::map::RoadClass;

use crate::osm::Tags;
use crate::tags::tag;

pub fn road_class(tags: &Tags) -> Option<RoadClass> {
    let class = RoadClass::from_highway(tag(tags, "highway")?)?;
    if tag(tags, "area") == Some("yes") {
        return None;
    }
    if matches!(tag(tags, "access"), Some("private" | "no")) {
        return None;
    }
    if class == RoadClass::Service && is_excluded_service(tags) {
        return None;
    }
    Some(class)
}

fn is_excluded_service(tags: &Tags) -> bool {
    matches!(
        tag(tags, "service"),
        Some("parking_aisle" | "driveway" | "drive-through" | "emergency_access")
    )
}
