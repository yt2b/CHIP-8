pub struct FrequencyTimer {
    elapsed_ms: f32,
    frame_time: f32,
}

impl FrequencyTimer {
    pub fn new(hz: u32) -> Self {
        assert!(hz > 0, "Frequency must be greater than 0");
        Self {
            elapsed_ms: 0.0,
            frame_time: 1000.0 / (hz as f32),
        }
    }

    pub fn update(&mut self, elapsed_ms: f32) -> usize {
        let mut count = 0;
        self.elapsed_ms += elapsed_ms;
        while self.elapsed_ms >= self.frame_time {
            self.elapsed_ms -= self.frame_time;
            count += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frequency_timer() {
        let mut timer = FrequencyTimer::new(60);
        assert_eq!(timer.update(16.67), 1);
        assert_eq!(timer.update(33.34), 2);
        assert_eq!(timer.update(10.0), 0);
        assert_eq!(timer.update(7.0), 1);
    }
}
