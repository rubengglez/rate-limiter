use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};
use tokio::time;

const MAX_TOKENS_PER_IP: i32 = 10;

struct TokenBucket {
    tokens_per_ip: Lazy<HashMap<String, i32>>,
}

static INSTANCE: Mutex<TokenBucket> = Mutex::new(TokenBucket {
    tokens_per_ip: Lazy::new(|| HashMap::new()),
});

pub async fn add_tokens() {
    let mut interval = time::interval(time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        reset_tokens().expect("Add tokens to bucket");
    }
}

pub fn use_token(ip: String) -> Result<(), String> {
    let mut instance = INSTANCE.lock().unwrap();
    let value = instance
        .tokens_per_ip
        .entry(ip)
        .or_insert(MAX_TOKENS_PER_IP);
    *value -= 1;

    if *value <= 0 {
        return Err(String::from("Invalid"));
    }
    Ok(())
}

fn reset_tokens() -> Result<(), ()> {
    for value in INSTANCE.lock().unwrap().tokens_per_ip.values_mut() {
        *value = MAX_TOKENS_PER_IP;
    }
    Ok(())
}
