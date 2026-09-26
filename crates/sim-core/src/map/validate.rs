use super::{MapData, MapError};

pub fn validate(map: &MapData) -> Result<(), MapError> {
    validate_nodes(map)?;
    validate_roads(map)?;
    validate_points(map)?;
    validate_names(map)?;
    validate_turn_bans(map)?;
    validate_areas(map)
}

fn invalid(message: String) -> Result<(), MapError> {
    Err(MapError::Invalid(message))
}

fn validate_nodes(map: &MapData) -> Result<(), MapError> {
    let nodes = &map.nodes;
    if nodes.y.len() != nodes.x.len() || nodes.control.len() != nodes.x.len() {
        return invalid("node table lengths differ".to_string());
    }
    Ok(())
}

fn validate_roads(map: &MapData) -> Result<(), MapError> {
    let roads = &map.roads;
    let count = map.road_count();
    let lengths = [
        roads.to.len(),
        roads.class.len(),
        roads.lanes_forward.len(),
        roads.lanes_backward.len(),
        roads.speed_kph.len(),
        roads.layer.len(),
        roads.name.len(),
        roads.roundabout.len(),
    ];
    if lengths.iter().any(|&len| len != count) {
        return invalid("road table lengths differ".to_string());
    }
    (0..count).try_for_each(|road| validate_road(map, road))
}

fn validate_road(map: &MapData, road: usize) -> Result<(), MapError> {
    let roads = &map.roads;
    let node_count = map.node_count();
    let (from, to) = (roads.from[road] as usize, roads.to[road] as usize);
    if from >= node_count || to >= node_count {
        return invalid(format!("road {road} references a missing node"));
    }
    if from == to {
        return invalid(format!("road {road} starts and ends at node {from}"));
    }
    if roads.lanes_forward[road] == 0 && roads.lanes_backward[road] == 0 {
        return invalid(format!("road {road} has no lanes"));
    }
    Ok(())
}

fn validate_points(map: &MapData) -> Result<(), MapError> {
    let starts = &map.roads.point_start;
    let point_count = map.points.x.len();
    if map.points.y.len() != point_count {
        return invalid("point table lengths differ".to_string());
    }
    if starts.len() != map.road_count() + 1 || starts[0] != 0 {
        return invalid("point_start must have road_count + 1 entries starting at 0".to_string());
    }
    if starts
        .windows(2)
        .any(|pair| pair[1] < pair[0].saturating_add(2))
    {
        return invalid("every road needs at least 2 points".to_string());
    }
    if starts[starts.len() - 1] as usize != point_count {
        return invalid("point_start must end at the point count".to_string());
    }
    Ok(())
}

fn validate_names(map: &MapData) -> Result<(), MapError> {
    let name_count = map.names.len();
    if map
        .roads
        .name
        .iter()
        .any(|&name| name as usize >= name_count)
    {
        return invalid("road name index out of range".to_string());
    }
    Ok(())
}

fn validate_turn_bans(map: &MapData) -> Result<(), MapError> {
    let (node_count, road_count) = (map.node_count(), map.road_count());
    let out_of_range = map.turn_bans.iter().any(|ban| {
        ban.via_node as usize >= node_count
            || ban.from_road as usize >= road_count
            || ban.to_road as usize >= road_count
    });
    if out_of_range {
        return invalid("turn ban index out of range".to_string());
    }
    Ok(())
}

fn validate_areas(map: &MapData) -> Result<(), MapError> {
    let areas = &map.areas;
    let starts = &areas.ring_start;
    if areas.kind.len() + 1 != starts.len() || starts[0] != 0 {
        return invalid("ring_start must have ring_count + 1 entries starting at 0".to_string());
    }
    if starts
        .windows(2)
        .any(|pair| pair[1] < pair[0].saturating_add(3))
    {
        return invalid("every area ring needs at least 3 points".to_string());
    }
    if areas.y.len() != areas.x.len() || starts[starts.len() - 1] as usize != areas.x.len() {
        return invalid("ring_start must end at the area point count".to_string());
    }
    Ok(())
}
