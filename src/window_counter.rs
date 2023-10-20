use chrono::prelude::*;
use std::collections::HashMap;

use crate::domain::Limiter;

const MAX_REQUESTS_PER_WINDOW: u8 = 10;

#[derive(Debug, Clone)]
struct Info {
    tokens: u8,
    timestamp: DateTime<Utc>,
}

impl Info {
    fn new() -> Info {
        Info {
            tokens: 1,
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowLimiter {
    counter: HashMap<String, Info>,
}

impl WindowLimiter {
    pub fn new() -> WindowLimiter {
        WindowLimiter {
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
                    value.tokens += 1;
                } else {
                    value.timestamp = current_date_time;
                    value.tokens = 1;
                }
            }
            None => {
                self.counter.insert(ip.to_string(), Info::new());
            }
        }
    }
}

impl Limiter for WindowLimiter {
    fn use_token(&mut self, ip: String) -> Result<(), String> {
        self.increase_counter(&ip);

        let current_counter = self.counter.get(&ip).unwrap();

        if current_counter.tokens > MAX_REQUESTS_PER_WINDOW {
            return Err("window error".to_string());
        }

        Ok(())
    }
}
