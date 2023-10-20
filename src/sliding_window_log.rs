use chrono::prelude::*;
use std::collections::HashMap;

use crate::domain::Limiter;

const MAX_REQUESTS_PER_SLIDING_WINDOW_LOG: u8 = 10;
const THRESHOLD_IN_MICROS: i64 = 60 * 1000 * 1000;

#[derive(Debug, Clone)]
pub struct SlidingWindowLog {
    counter: HashMap<String, Vec<i64>>,
}

impl SlidingWindowLog {
    pub fn new() -> SlidingWindowLog {
        SlidingWindowLog {
            counter: HashMap::new(),
        }
    }
}

impl Limiter for SlidingWindowLog {
    fn use_token(&mut self, ip: String) -> Result<(), String> {
        match self.counter.get_mut(&ip) {
            Some(requests) => {
                let now = Utc::now();
                let limit = now.timestamp_micros() - THRESHOLD_IN_MICROS;
                // Improve performance here. Right now is O(N)
                requests.retain(|x| limit < *x);
                if requests.len() >= MAX_REQUESTS_PER_SLIDING_WINDOW_LOG as usize {
                    return Err(String::from("Limit reached"));
                }

                requests.push(now.timestamp_micros());
            }
            None => {
                self.counter
                    .insert(ip.to_string(), vec![Utc::now().timestamp()]);
            }
        }
        Ok(())
    }
}
