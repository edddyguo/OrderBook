use serde::{Serialize,Deserialize};

#[derive(Serialize)]
pub struct Trade {
    pub id : String,
    pub price : f32,
    pub amount : f32,
    pub taker_side : String,
    pub updated_at : u32,
}
//{"id":"BTC-USDT","price":1000.0,"amount":10.1,"taker_side":"buy","updated_at":1644287259123},