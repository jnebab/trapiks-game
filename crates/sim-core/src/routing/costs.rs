use crate::consts::{EMA_ALPHA, HEURISTIC_SPEED, MIN_LINK_SPEED};
use crate::network::{LinkId, Network};

const MIN_DRIVE_LEN: f64 = 0.1;

#[derive(Clone, Debug, Default)]
pub struct LinkCosts {
    pub free_speed: Vec<f64>,
    pub ema_speed: Vec<f64>,
    pub drive_len: Vec<f64>,
    travel: Vec<f64>,
    window: Vec<(f64, u32)>,
    heuristic_speed: f64,
    inverse_heuristic_speed: f64,
}

impl LinkCosts {
    pub fn build(network: &Network) -> LinkCosts {
        let mut costs = LinkCosts {
            heuristic_speed: HEURISTIC_SPEED,
            ..LinkCosts::default()
        };
        costs.grow(network);
        costs
    }

    pub fn grow(&mut self, network: &Network) {
        for link in self.free_speed.len()..network.link_count() {
            let link = link as LinkId;
            let speed = network.link_speed(link);
            let (start, end) = network.link_span(link);
            self.free_speed.push(speed);
            self.ema_speed.push(speed);
            let len = (end - start).max(MIN_DRIVE_LEN);
            self.drive_len.push(len);
            self.travel.push(len / speed.max(MIN_LINK_SPEED));
            self.window.push((0.0, 0));
            self.heuristic_speed = self.heuristic_speed.max(speed);
        }
        self.inverse_heuristic_speed = 1.0 / self.heuristic_speed;
    }

    pub fn heuristic_speed(&self) -> f64 {
        self.heuristic_speed
    }

    pub fn inverse_heuristic_speed(&self) -> f64 {
        self.inverse_heuristic_speed
    }

    pub fn travel_time(&self, link: LinkId) -> f64 {
        self.travel
            .get(link as usize)
            .copied()
            .unwrap_or(f64::INFINITY)
    }

    pub fn free_time(&self, link: LinkId) -> f64 {
        let index = link as usize;
        match (self.drive_len.get(index), self.free_speed.get(index)) {
            (Some(&len), Some(&speed)) => len / speed.max(MIN_LINK_SPEED),
            _ => f64::INFINITY,
        }
    }

    pub fn accumulate(&mut self, link: LinkId, v: f64) {
        if let Some((sum, count)) = self.window.get_mut(link as usize) {
            *sum += v;
            *count += 1;
        }
    }

    pub fn refresh(&mut self) {
        for index in 0..self.window.len() {
            let free = self.free_speed[index];
            let (sum, count) = self.window[index];
            let sample = if count == 0 {
                free
            } else {
                (sum / f64::from(count)).min(free)
            };
            let ema = (1.0 - EMA_ALPHA) * self.ema_speed[index] + EMA_ALPHA * sample;
            self.ema_speed[index] = ema;
            self.travel[index] = self.drive_len[index] / ema.max(MIN_LINK_SPEED);
            self.window[index] = (0.0, 0);
        }
    }
}
