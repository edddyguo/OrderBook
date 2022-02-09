use serde::{Serialize,Deserialize};

#[derive(Deserialize, Debug)]
pub enum Side {
    #[serde(rename = "buy")]
    BUY,
    #[serde(rename = "sell")]
    SELL,
}

#[derive(Clone,Serialize)]
pub struct Depth {
    pub asks: Vec<(f32,f32)>,
    pub bids: Vec<(f32,f32)>,
}

impl Depth {
    pub fn sort(&mut self){
        self.asks.sort_by(|a,b|{
            a.0.partial_cmp(&b.0).unwrap()
        });
        self.asks.reverse();
        self.bids.sort_by(|a,b|{
            a.0.partial_cmp(&b.0).unwrap()
        });
    }
}