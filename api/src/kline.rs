use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Kline {
    time: u32,
    open: f32,
    close: f32,
    high: f32,
    low: f32,
    volume: f32,
}