use crate::rng::Pcg32;

const SECONDS_PER_HOUR: f64 = 3600.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TripClock {
    next_s: f64,
    rate: f64,
}

impl TripClock {
    pub fn new(vehicles_per_hour: f64, rng: &mut Pcg32) -> TripClock {
        let mut clock = TripClock {
            next_s: f64::INFINITY,
            rate: 0.0,
        };
        clock.set_rate(vehicles_per_hour, 0.0, rng);
        clock
    }

    pub fn set_rate(&mut self, vehicles_per_hour: f64, time_s: f64, rng: &mut Pcg32) {
        self.rate = (vehicles_per_hour / SECONDS_PER_HOUR).max(0.0);
        self.next_s = f64::INFINITY;
        if self.rate > 0.0 {
            self.next_s = time_s + self.interval(rng);
        }
    }

    pub fn is_due(&self, time_s: f64) -> bool {
        self.next_s <= time_s
    }

    pub fn advance(&mut self, rng: &mut Pcg32) {
        self.next_s += self.interval(rng);
    }

    pub fn next_s(&self) -> f64 {
        self.next_s
    }

    fn interval(&self, rng: &mut Pcg32) -> f64 {
        -libm::log(1.0 - rng.next_f64()) / self.rate
    }
}
