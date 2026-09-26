mod append;
mod content_hash;
mod junction;
mod link;
mod mutate;
mod node;
mod region;
mod road;
mod setback;
mod signal;
mod spatial;

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};

use crate::consts::LANE_WIDTH;
use crate::geom::Vec2;
use crate::map::{MapData, TurnBan};

pub use junction::{Conflict, Junction, Movement, TurnKind, build_junction};
use junction::{turns, turns_ignoring_bans};
pub use link::{Direction, LinkId, direction_of, link_id, reverse, road_of};
use link::{arriving, departing};
pub use node::NodeStore;
pub use region::{Region, RegionMask};
pub use road::RoadStore;
pub use setback::setback;
pub use signal::{SignalCluster, SignalInputs, SignalState, Signals, Timing, cycle_ticks};
pub use spatial::SpatialGrid;

pub struct Network {
    pub roads: RoadStore,
    pub nodes: NodeStore,
    pub spatial: SpatialGrid,
    bans: BTreeSet<TurnBan>,
    timings: BTreeMap<u32, Timing>,
    signals: Signals,
    region: Option<RegionMask>,
    junctions: Vec<Option<Junction>>,
    spans: Vec<(f64, f64)>,
    version: u64,
    structure: u64,
    spatial_structure: u64,
    content: Cell<Option<(u64, u64)>>,
}

impl Network {
    pub fn from_map(map: &MapData) -> Network {
        let roads = RoadStore::from_map(map);
        let nodes = NodeStore::from_map(map);
        let spatial = SpatialGrid::build(&roads, &nodes);
        let mut network = Network {
            junctions: vec![None; nodes.count()],
            bans: map.turn_bans.iter().copied().collect(),
            timings: BTreeMap::new(),
            roads,
            nodes,
            spatial,
            signals: Signals::default(),
            region: None,
            spans: Vec::new(),
            version: 0,
            structure: 0,
            spatial_structure: 0,
            content: Cell::new(None),
        };
        network.rebuild_signals();
        network.compute_spans();
        network
    }

    fn compute_spans(&mut self) {
        self.spans = (0..self.link_count() as LinkId)
            .map(|link| self.measure_span(link))
            .collect();
    }

    fn refresh_spans_at(&mut self, node: u32) {
        let Some(incident) = self.nodes.roads.get(node as usize) else {
            return;
        };
        let links: Vec<LinkId> = incident
            .iter()
            .flat_map(|&road| [Direction::Forward, Direction::Backward].map(|d| link_id(road, d)))
            .collect();
        for link in links {
            let span = self.measure_span(link);
            if let Some(slot) = self.spans.get_mut(link as usize) {
                *slot = span;
            }
        }
    }

    pub fn set_region(&mut self, region: Option<Region>) {
        self.region = region.map(|r| RegionMask::build(self, r));
        self.rebuild_signals();
        self.junctions.iter_mut().for_each(|slot| *slot = None);
        self.compute_spans();
        self.version += 1;
    }

    fn rebuild_signals(&mut self) {
        let inputs = SignalInputs {
            roads: &self.roads,
            nodes: &self.nodes,
            spatial: &self.spatial,
            timings: &self.timings,
        };
        self.signals = Signals::build(&inputs, |road| self.is_road_active(road));
    }

    pub fn is_road_active(&self, road: u32) -> bool {
        let in_region = self
            .region
            .as_ref()
            .is_none_or(|mask| mask.active_road[road as usize]);
        self.roads.is_live(road) && in_region
    }

    pub fn is_link_active(&self, link: LinkId) -> bool {
        let road = road_of(link);
        (road as usize) < self.roads.count()
            && self.is_road_active(road)
            && self.link_lanes(link) > 0
    }

    pub fn link_lanes(&self, link: LinkId) -> u8 {
        link::lanes(&self.roads, link)
    }

    pub fn link_from(&self, link: LinkId) -> u32 {
        link::from_node(&self.roads, link)
    }

    pub fn link_to(&self, link: LinkId) -> u32 {
        link::to_node(&self.roads, link)
    }

    pub fn link_length(&self, link: LinkId) -> f64 {
        link::length(&self.roads, link)
    }

    pub fn link_speed(&self, link: LinkId) -> f64 {
        self.roads.speed[road_of(link) as usize]
    }

    pub fn incoming(&self, node: u32) -> impl Iterator<Item = LinkId> + '_ {
        self.incident(node)
            .map(move |road| arriving(&self.roads, road, node))
            .filter(|&link| self.is_link_active(link))
    }

    pub fn outgoing(&self, node: u32) -> impl Iterator<Item = LinkId> + '_ {
        self.incident(node)
            .map(move |road| departing(&self.roads, road, node))
            .filter(|&link| self.is_link_active(link))
    }

    fn incident(&self, node: u32) -> impl Iterator<Item = u32> + '_ {
        self.nodes.roads[node as usize].iter().copied()
    }

    pub fn active_degree(&self, node: u32) -> usize {
        self.nodes
            .active_degree(node, |road| self.is_road_active(road))
    }

    pub fn setback_at(&self, node: u32, road: u32) -> f64 {
        setback(
            &self.roads,
            &self.nodes,
            |r| self.is_road_active(r),
            node,
            road,
        )
    }

    pub fn link_span(&self, link: LinkId) -> (f64, f64) {
        match self.spans.get(link as usize) {
            Some(&span) => span,
            None => self.measure_span(link),
        }
    }

    fn measure_span(&self, link: LinkId) -> (f64, f64) {
        let road = road_of(link);
        let start = self.setback_at(self.link_from(link), road);
        let end = self.link_length(link) - self.setback_at(self.link_to(link), road);
        (start, end)
    }

    pub fn departing_link(&self, road: u32, node: u32) -> LinkId {
        departing(&self.roads, road, node)
    }

    pub fn centre_pose(&self, link: LinkId, s: f64) -> (Vec2, Vec2) {
        link::centre_pose(&self.roads, link, s)
    }

    pub fn lane_offset(&self, road: u32, lane: u8) -> f64 {
        self.roads.width(road) / 2.0 - (f64::from(lane) + 0.5) * LANE_WIDTH
    }

    pub fn link_pose(&self, link: LinkId, s: f64, lane: u8) -> (Vec2, Vec2) {
        let (pos, tangent) = self.centre_pose(link, s);
        let offset = self.lane_offset(road_of(link), lane);
        (pos + tangent.perp_right() * offset, tangent)
    }

    pub fn is_banned(&self, node: u32, from_road: u32, to_road: u32) -> bool {
        self.bans.contains(&TurnBan {
            via_node: node,
            from_road,
            to_road,
        })
    }

    pub fn ensure_junction(&mut self, node: u32) -> &Junction {
        let index = node as usize;
        let built = self.junctions[index]
            .take()
            .unwrap_or_else(|| build_junction(self, node));
        self.junctions[index].insert(built)
    }

    pub fn turns(&self, node: u32) -> Vec<(LinkId, LinkId, TurnKind)> {
        turns(self, node)
    }

    pub fn turns_ignoring_bans(&self, node: u32) -> Vec<(LinkId, LinkId, TurnKind)> {
        turns_ignoring_bans(self, node)
    }

    pub fn timing_override(&self, key: u32) -> Option<&Timing> {
        self.timings.get(&key)
    }

    pub fn link_count(&self) -> usize {
        self.roads.count() * 2
    }

    #[cfg(feature = "fixtures")]
    pub fn ban_turn_for_test(&mut self, node: u32, from_road: u32, to_road: u32) {
        self.bans.insert(TurnBan {
            via_node: node,
            from_road,
            to_road,
        });
        self.invalidate_node(node);
        self.version += 1;
    }

    pub fn junction(&self, node: u32) -> Option<&Junction> {
        self.junctions.get(node as usize).and_then(Option::as_ref)
    }

    pub fn invalidate_node(&mut self, node: u32) {
        if let Some(slot) = self.junctions.get_mut(node as usize) {
            *slot = None;
        }
        self.refresh_spans_at(node);
    }

    pub fn build_all_junctions(&mut self) {
        for node in 0..self.nodes.count() as u32 {
            self.ensure_junction(node);
        }
    }

    pub fn signal_state(&self, link: LinkId, tick: u64) -> Option<SignalState> {
        self.signals.state(link, tick)
    }

    pub fn signals(&self) -> &Signals {
        &self.signals
    }

    pub fn region(&self) -> Option<&RegionMask> {
        self.region.as_ref()
    }

    pub fn version(&self) -> u64 {
        self.version
    }
}
