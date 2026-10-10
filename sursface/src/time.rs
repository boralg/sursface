use lazy_static::lazy_static;
use web_time::{Duration, Instant};

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

pub struct Time {
    last_timestamp: Instant,
    pub dt: Duration,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            last_timestamp: *START_TIME,
            dt: Default::default(),
        }
    }
}

impl Time {
    pub fn update(&mut self) {
        let now = Instant::now();

        self.dt = self.last_timestamp.elapsed();
        self.last_timestamp = now;
    }

    pub fn since_start() -> Duration {
        Instant::now().duration_since(*START_TIME)
    }
}
