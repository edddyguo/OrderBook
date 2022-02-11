use rustc_serialize::json;
use serde::Serialize;
use ethers::{prelude::*,types::{U256}};



#[derive(RustcEncodable, Clone, Serialize)]
pub struct EventOrder {
    pub market_id: String,
    pub side: String,
    pub price: f64,
    pub amount: f64,
}

#[derive(Clone, Serialize,Debug)]
pub struct EngineOrder {
    pub id: String,
    pub side: String,
    pub price: U256,
    pub amount: U256,
    pub created_at: u64,
}

pub fn r#match(taker_order: EngineOrder){
    todo!()
}

pub fn cancel(){
    todo!()
}

pub fn flush(){
    todo!()
}