use serde::{Deserialize, Serialize};

use common::types::order::Side as OrderSide;

#[derive(Serialize)]
pub struct Trade {
    pub id: String,
    pub transaction_hash: String,
    pub market_id: String,
    pub price: f64,
    pub amount: f64,
    pub height: u32,
    pub taker_side: OrderSide,
    pub updated_at: u64,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct Trade2 {
    pub id: String,
    pub market_id: String,
    pub price: f64,
    pub amount: f64,
    pub height: u32,
    pub status: String,
    pub taker_side: OrderSide,
    pub updated_at: u64,
}
//{"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
