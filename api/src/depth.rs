use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub struct Depth {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

impl Depth {
    pub fn sort(&mut self) {
        self.asks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.asks.reverse();
        self.bids.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }
}
