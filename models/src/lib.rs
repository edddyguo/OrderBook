pub mod api;
pub mod chain;
pub mod order;
pub mod trade;

#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use postgres::{Client, Error, NoTls, Row};
use std::any::Any;
use std::env;

use std::fmt::Debug;

use std::sync::Mutex;
use anyhow::{anyhow, Result};


extern crate chrono;
extern crate postgres;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use chrono::Local;
use serde_json::error::Category::Data;
use common::types::*;
use crate::trade::TradeInfo;
use crate::order::OrderInfo;

lazy_static! {
    static ref CLIENTDB: Mutex<postgres::Client> = Mutex::new(connetDB().unwrap());
}

pub fn restartDB() -> bool {
    let now = Local::now();
    println!("restart postgresql {:?}", now);
    // let client =  connetDB();
    if let Some(client) = connetDB() {
        *crate::CLIENTDB.lock().unwrap() = client;
        return true;
    }
    false
}

fn connetDB() -> Option<postgres::Client> {
    let dbname = match env::var_os("CHEMIX_MODE") {
        None => "chemix_local".to_string(),
        Some(mist_mode) => {
            format!("chemix_{}", mist_mode.into_string().unwrap())
        }
    };

    let url = format!(
        "host=localhost user=postgres port=5432 password=postgres dbname={}",
        dbname
    );

    match Client::connect(&url, NoTls) {
        Ok(client) => {
            eprintln!("connect postgresql successfully");
            Some(client)
        }
        Err(error) => {
            eprintln!("connect postgresql failed,{:?}", error);
            None
        }
    }
}


pub fn query(raw_sql: &str) -> anyhow::Result<Vec<Row>>{
    let mut try_times = 5;
    loop {
        match  crate::CLIENTDB.lock().unwrap().query(raw_sql, &[]) {
            Ok(data) => {
                return Ok(data);
            }
            Err(error) => {
                if try_times == 0 {
                    //Err(anyhow!("Missing attribute: {}", missing));
                    return Err(anyhow!("retry query failed"));
                }else {
                    info!("error {:?}",error);
                    crate::restartDB();
                    try_times -= 1;
                    continue;
                }
            }
        }
    }
}

pub fn execute(raw_sql: &str) -> anyhow::Result<u64>{
    let mut try_times = 5;
    loop {
        match  crate::CLIENTDB.lock().unwrap().execute(raw_sql, &[]) {
            Ok(data) => {
                return Ok(data);
            }
            Err(_) => {
                if try_times == 0 {
                    //Err(anyhow!("Missing attribute: {}", missing));
                    return Err(anyhow!("retry execute failed"));
                }else {
                    crate::restartDB();
                    try_times -= 1;
                    continue;
                }
            }
        }
    }
}

pub trait FormatSql {
    fn string4sql(&self) -> String;
}

impl FormatSql for String {
    fn string4sql(&self) -> String {
        format!("'{}'", self)
    }
}

pub fn struct2array<T: Any + Debug>(value: &T) -> Vec<String> {
    let mut trade_vec: Vec<String> = vec![];
    let value = value as &dyn Any;
    match value.downcast_ref::<TradeInfo>() {
        Some(trade) => {
            trade_vec.push(trade.id.string4sql());
            trade_vec.push(trade.transaction_id.to_string());
            trade_vec.push(trade.transaction_hash.string4sql());
            trade_vec.push(trade.status.as_str().to_string().string4sql());
            trade_vec.push(trade.market_id.string4sql());
            trade_vec.push(trade.maker.string4sql());
            trade_vec.push(trade.taker.string4sql());
            trade_vec.push(trade.price.to_string());
            trade_vec.push(trade.amount.to_string());
            trade_vec.push(trade.taker_side.as_str().to_string().string4sql());
            trade_vec.push(trade.maker_order_id.string4sql());
            trade_vec.push(trade.taker_order_id.string4sql());
            trade_vec.push(trade.updated_at.string4sql());
            trade_vec.push(trade.created_at.string4sql());
        }
        None => (),
    };
    match value.downcast_ref::<order::OrderInfo>() {
        Some(trade) => {
            trade_vec.push(trade.id.string4sql());
            trade_vec.push(trade.index.to_string());
            trade_vec.push(trade.market_id.string4sql());
            trade_vec.push(trade.account.string4sql());
            trade_vec.push(trade.side.as_str().to_string().string4sql());
            trade_vec.push(trade.price.to_string());
            trade_vec.push(trade.amount.to_string());
            trade_vec.push(trade.status.as_str().to_string().string4sql());
            trade_vec.push(trade.available_amount.to_string());
            trade_vec.push(trade.matched_amount.to_string());
            trade_vec.push(trade.canceled_amount.to_string());
            trade_vec.push(trade.updated_at.string4sql());
            trade_vec.push(trade.created_at.string4sql());
        }
        None => (),
    };
    trade_vec
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
