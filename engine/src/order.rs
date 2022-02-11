use rustc_serialize::json;
use serde::Serialize;


#[derive(RustcEncodable, Clone, Serialize)]
pub struct EventOrder {
    pub market_id: String,
    pub side: String,
    pub price: f32,
    pub amount: f32,
}

#[derive(RustcEncodable, Clone, Serialize)]
pub struct EngineOrder {
    pub id: String,
    pub side: String,
    pub price: f32,
    pub amount: f32,
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