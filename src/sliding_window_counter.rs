use chrono::prelude::*;
use std::collections::HashMap;

use crate::domain::Limiter;

const MAX_REQUESTS_PER_WINDOW: u32 = 10;

#[derive(Debug, Clone)]
struct Info {
    current_window: u8,
    previous_window: u8,
    timestamp: DateTime<Utc>,
}

impl Info {
    fn new() -> Info {
        Info {
            current_window: 1,
            previous_window: 1,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SlidingWindowCounter {
    counter: HashMap<String, Info>,
}

impl SlidingWindowCounter {
    pub fn new() -> SlidingWindowCounter {
        SlidingWindowCounter {
            counter: HashMap::new(),
        }
    }

    fn increase_counter(&mut self, ip: &str) {
        let current_date_time = Utc::now();
        match self.counter.get_mut(ip) {
            Some(value) => {
                if value.timestamp.hour() == current_date_time.hour()
                    && value.timestamp.minute() == current_date_time.minute()
                {
                    value.current_window += 1;
                } else {
                    value.previous_window = value.current_window;
                    value.current_window = 1;
                }
                value.timestamp = current_date_time;
            }
            None => {
                self.counter.insert(ip.to_string(), Info::new());
            }
        }
    }
}

impl Limiter for SlidingWindowCounter {
    fn use_token(&mut self, ip: String) -> Result<(), String> {
        self.increase_counter(&ip);

        let current_counter = self.counter.get(&ip).unwrap();

        let percentage_time_current_window = current_counter.timestamp.second() * 100 / 60;

        let tokens = (((100 - percentage_time_current_window) as f64) / 100 as f64)
            * current_counter.previous_window as f64
            + current_counter.current_window as f64;

        if tokens > MAX_REQUESTS_PER_WINDOW as f64 {
            return Err("window error".to_string());
        }

        Ok(())
    }
}
