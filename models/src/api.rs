use postgres::{config::Config, error::Error, row::SimpleQueryRow, Client, NoTls};

extern crate rustc_serialize;

use rustc_serialize::json;

use std::ops::Mul;
//#[derive(Serialize)]
use serde::Serialize;

#[derive(Serialize, Debug, Default)]
pub struct MarketInfo {
    pub id: String,
    base_token_address: String,
    base_token_symbol: String,
    quote_token_address: String,
    quote_token_symbol: String,
    matched_address: String,
}

pub fn list_markets() -> Vec<MarketInfo> {
    let sql = "select id,base_token_address,base_token_symbol,quote_token_address,quote_token_symbol,matched_address from chemix_markets where online=true";
    let mut markets: Vec<MarketInfo> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    if let Err(err) = result {
        println!("get_active_address_num failed {:?}", err);
        if !crate::restartDB() {
            return markets;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    }
    let rows = result.unwrap();
    for row in rows {
        let info = MarketInfo {
            id: row.get(0),
            base_token_address: row.get(1),
            base_token_symbol: row.get(2),
            quote_token_address: row.get(3),
            quote_token_symbol: row.get(4),
            matched_address: row.get(5),
        };
        markets.push(info);
    }
    markets
}