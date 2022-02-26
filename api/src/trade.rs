use serde::Serialize;
use common::types::order::Status as OrderStatus;
use common::types::trade::Status as TradeStatus;
use common::types::order::Side as OrderSide;

#[derive(Serialize)]
pub struct Trade {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub height: u32,
    pub taker_side: OrderSide,
    pub updated_at: u64,
}
//{"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},
