// use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::domain::{Limiter, Stoppable};

const MAX_TOKENS_PER_IP: i32 = 10;
const TIME_TO_ADD_TOKENS: u8 = 10;

pub struct TokenBucket {
    tokens_per_ip: Arc<Mutex<HashMap<String, i32>>>,
    tx: Mutex<Option<Sender<bool>>>,
}

impl TokenBucket {
    pub fn new() -> TokenBucket {
        TokenBucket {
            tokens_per_ip: Arc::new(Mutex::new(HashMap::new())),
            tx: Mutex::new(None),
        }
    }

    pub fn start(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.tx = Mutex::new(Some(tx.clone()));
        let tokens = self.tokens_per_ip.clone();

        let add_tokens = move || {
            for (_, value) in tokens.lock().unwrap().iter_mut() {
                *value = MAX_TOKENS_PER_IP;
            }
        };

        thread::spawn(move || loop {
            tx.send(true).unwrap();
            thread::sleep(Duration::from_secs(TIME_TO_ADD_TOKENS as u64));
        });

        thread::spawn(move || {
            for _received in rx {
                add_tokens();
            }
        });
    }
}

impl Stoppable for TokenBucket {
    fn stop(&self) {
        let tx = self.tx.lock().unwrap();
        match tx.deref() {
            None => (),
            Some(x) => drop(x),
        }
    }
}

impl Limiter for TokenBucket {
    fn use_token(&mut self, ip: String) -> Result<(), String> {
        let mut instance = self.tokens_per_ip.lock().unwrap();
        let value = instance.entry(ip).or_insert(MAX_TOKENS_PER_IP);
        *value -= 1;

        if *value <= 0 {
            return Err(String::from("Invalid"));
        }
        Ok(())
    }
}
