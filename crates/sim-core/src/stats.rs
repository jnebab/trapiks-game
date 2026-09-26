use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::consts::DT;

const SECONDS_PER_HOUR: f64 = 3600.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StatsWindow {
    pub start_tick: u64,
    pub created: u64,
    pub arrivals: u64,
    pub travel_sum: f64,
    pub delay_sum: f64,
    pub accrued_delay: f64,
    pub spawned: u64,
    pub unserved: u64,
    pub stranded: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct StatsSnapshot {
    pub tick: u64,
    pub sim_time_s: f64,
    pub active: u32,
    pub created: u64,
    pub spawned: u64,
    pub arrivals: u64,
    pub unserved: u64,
    pub stranded: u64,
    pub mean_travel_s: f64,
    pub mean_delay_s: f64,
    pub accrued_delay_s: f64,
    pub throughput_per_hour: f64,
    pub mean_speed: f64,
    pub stopped_share: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LiveSpeeds {
    pub active: u32,
    pub speed_sum: f64,
    pub stopped: u32,
}

impl StatsWindow {
    pub fn reset(&mut self, tick: u64) {
        *self = StatsWindow {
            start_tick: tick,
            ..StatsWindow::default()
        };
    }

    pub fn record_arrival(&mut self, travel: f64, free_flow: f64) {
        self.arrivals += 1;
        self.travel_sum += travel;
        self.delay_sum += (travel - free_flow).max(0.0);
    }

    pub fn snapshot(&self, tick: u64, live: LiveSpeeds) -> StatsSnapshot {
        StatsSnapshot {
            tick,
            sim_time_s: tick as f64 * DT,
            active: live.active,
            created: self.created,
            spawned: self.spawned,
            arrivals: self.arrivals,
            unserved: self.unserved,
            stranded: self.stranded,
            mean_travel_s: per(self.travel_sum, self.arrivals),
            mean_delay_s: per(self.delay_sum, self.arrivals),
            accrued_delay_s: self.accrued_delay,
            throughput_per_hour: self.throughput(tick),
            mean_speed: per(live.speed_sum, u64::from(live.active)),
            stopped_share: per(f64::from(live.stopped), u64::from(live.active)),
        }
    }

    fn throughput(&self, tick: u64) -> f64 {
        if tick <= self.start_tick {
            return 0.0;
        }
        let hours = (tick - self.start_tick) as f64 * DT / SECONDS_PER_HOUR;
        self.arrivals as f64 / hours
    }
}

fn per(sum: f64, count: u64) -> f64 {
    if count == 0 {
        return 0.0;
    }
    sum / count as f64
}
