pub mod api;
pub mod chain;
pub mod order;
pub mod trade;

#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use postgres::{Client, NoTls};
use std::any::Any;
use std::env;

use std::fmt::Debug;

use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate postgres;

use chrono::Local;

use crate::order::{OrderInfo, Side};
use crate::trade::TradeInfo;

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
            let side = match trade.taker_side {
                Side::Buy => "buy",
                Side::Sell => "sell",
            };

            trade_vec.push(trade.id.string4sql());
            trade_vec.push(trade.transaction_id.to_string());
            trade_vec.push(trade.transaction_hash.string4sql());
            trade_vec.push(trade.status.string4sql());
            trade_vec.push(trade.market_id.string4sql());
            trade_vec.push(trade.maker.string4sql());
            trade_vec.push(trade.taker.string4sql());
            trade_vec.push(trade.price.to_string());
            trade_vec.push(trade.amount.to_string());
            trade_vec.push(side.to_string().string4sql());
            trade_vec.push(trade.maker_order_id.string4sql());
            trade_vec.push(trade.taker_order_id.string4sql());
            trade_vec.push(trade.updated_at.string4sql());
            trade_vec.push(trade.created_at.string4sql());
        }
        None => (),
    };
    match value.downcast_ref::<OrderInfo>() {
        Some(trade) => {
            trade_vec.push(trade.id.string4sql());
            trade_vec.push(trade.market_id.string4sql());
            trade_vec.push(trade.account.string4sql());
            trade_vec.push(trade.side.string4sql());
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
