use std::collections::{BTreeMap, VecDeque};

use crate::network::LinkId;
use crate::vehicle::SpawnError;

use super::Trip;

#[derive(Clone, Debug, PartialEq)]
pub struct RoutedTrip {
    pub trip: Trip,
    pub route: Vec<LinkId>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QueueOutcome {
    pub spawned: u64,
    pub unserved: u64,
}

#[derive(Clone, Debug, Default)]
pub struct LinkQueues {
    queues: BTreeMap<LinkId, VecDeque<RoutedTrip>>,
}

impl LinkQueues {
    pub fn push(&mut self, routed: RoutedTrip) {
        self.queues
            .entry(routed.trip.from)
            .or_default()
            .push_back(routed);
    }

    pub fn len(&self) -> usize {
        self.queues.values().map(VecDeque::len).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.queues.is_empty()
    }

    pub fn trips(&self) -> impl Iterator<Item = &Trip> + '_ {
        self.queues.values().flatten().map(|routed| &routed.trip)
    }

    pub fn forget_links_from(&mut self, limit: LinkId) -> u64 {
        let before = self.len();
        for queue in self.queues.values_mut() {
            queue.retain(|routed| routed.route.iter().all(|&link| link < limit));
        }
        self.queues.retain(|_, queue| !queue.is_empty());
        (before - self.len()) as u64
    }

    pub fn step(
        &mut self,
        tick: u64,
        mut spawn: impl FnMut(&RoutedTrip) -> Result<u32, SpawnError>,
    ) -> QueueOutcome {
        let mut outcome = QueueOutcome::default();
        for queue in self.queues.values_mut() {
            step_queue(queue, tick, &mut spawn, &mut outcome);
        }
        self.queues.retain(|_, queue| !queue.is_empty());
        outcome
    }
}

fn step_queue(
    queue: &mut VecDeque<RoutedTrip>,
    tick: u64,
    spawn: &mut impl FnMut(&RoutedTrip) -> Result<u32, SpawnError>,
    outcome: &mut QueueOutcome,
) {
    while queue
        .front()
        .is_some_and(|routed| routed.trip.is_expired(tick))
    {
        queue.pop_front();
        outcome.unserved += 1;
    }
    let Some(front) = queue.front() else {
        return;
    };
    match spawn(front) {
        Err(SpawnError::Blocked) => {}
        Ok(_) => {
            queue.pop_front();
            outcome.spawned += 1;
        }
        Err(_) => {
            queue.pop_front();
            outcome.unserved += 1;
        }
    }
}
