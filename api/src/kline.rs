use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Kline {
    time: u32,
    open: f64,
    close: f64,
    high: f64,
    low: f64,
    volume: f64,
}
