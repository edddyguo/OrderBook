use serde::{Serialize};
use chemix_models::order::Side;

#[derive(Serialize)]
pub struct Trade {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub taker_side: Side,
    pub updated_at: u64,
}
//{"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
