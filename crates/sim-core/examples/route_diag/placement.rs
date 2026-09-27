use trapiks_sim_core::geom::Vec2;
use trapiks_sim_core::network::LinkId;
use trapiks_sim_core::routing::largest_component_members;
use trapiks_sim_core::sim::Sim;

pub fn report(sim: &Sim) {
    let network = sim.network();
    let graph = sim.route_graph();
    let core = largest_component_members(graph, |link| network.is_link_active(link));
    let (centre, reach) = extent(sim);
    for &link in sim.landmarks().links() {
        let point = network.nodes.pos[network.link_to(link) as usize];
        let offset = point - centre;
        let degrees = libm::atan2(offset.y, offset.x).to_degrees();
        println!(
            "landmark {link}: angle {degrees:.0} deg, radius {:.2} of max, in scc {}",
            offset.length() / reach,
            core[link as usize]
        );
    }
}

fn extent(sim: &Sim) -> (Vec2, f64) {
    let network = sim.network();
    let active: Vec<Vec2> = (0..network.link_count() as LinkId)
        .filter(|&link| network.is_link_active(link))
        .map(|link| network.nodes.pos[network.link_to(link) as usize])
        .collect();
    let (min, max) = active.iter().fold(
        (Vec2::new(f64::MAX, f64::MAX), Vec2::new(f64::MIN, f64::MIN)),
        |(lo, hi), p| {
            (
                Vec2::new(lo.x.min(p.x), lo.y.min(p.y)),
                Vec2::new(hi.x.max(p.x), hi.y.max(p.y)),
            )
        },
    );
    let centre = (min + max) * 0.5;
    let reach = active
        .iter()
        .map(|p| p.distance(centre))
        .fold(0.0, f64::max);
    (centre, reach.max(1.0))
}
