use rand::Rng;
use rand::distr::{Alphanumeric, SampleString};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time moved backwards")
        .as_secs()
}

pub fn current_unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time moved backwards")
        .as_millis()
}

pub fn generate_ms_token(length: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, length)
}

pub fn generate_verify_fp() -> String {
    let mut rng = rand::rng();
    let random_str = Alphanumeric.sample_string(&mut rng, 32).to_lowercase();
    format!("verify_{}", random_str)
}

pub fn generate_numeric_id(length: usize) -> String {
    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range(0..10).to_string())
        .collect()
}
