use std::time::Duration;

pub struct Ticker {
    pub interval: Duration,
    pub update: fn(),
}
