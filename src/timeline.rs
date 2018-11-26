#![allow(dead_code)]

use std::time::{Duration, Instant};

pub struct Timeline {
    start_time: Instant,
    previous_frame_time: Instant,
    previous_frame_duration: Duration,
}

impl Timeline {
    pub fn new() -> Timeline {
        Timeline {
            start_time: Instant::now(),
            previous_frame_time: Instant::now(),
            previous_frame_duration: Duration::from_secs(0),
        }
    }

    /// Notify the timeline that we've ended the current frame and proceeding to the next.
    pub fn next_frame(&mut self) -> &mut Self {
        let now = Instant::now();
        let duration = now.duration_since(self.previous_frame_time);
        self.previous_frame_time = now;
        self.previous_frame_duration = duration;
        self
    }

    /// Returns the duration of the last frame
    pub fn previous_frame_duration(&self) -> Duration {
        self.previous_frame_duration
    }

    /// Returns the duration of the last frame in fractional seconds
    pub fn previous_frame_time(&self) -> f32 {
        self.previous_frame_duration.as_secs() as f32
            + (f64::from(self.previous_frame_duration.subsec_nanos()) * 1e-9) as f32
    }
}
