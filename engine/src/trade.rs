use std::borrow::BorrowMut;
use std::collections::HashMap;
use rustc_serialize::json;
use serde::Serialize;
use std::ops::{Deref, Index, Sub};
use crate::{AddBook, LastTrade, EngineBook, AddBook2, LastTrade2};
//use ethers::{prelude::*,types::{U256}};
use serde::Deserialize;
use chemix_utils::algorithm::sha256;
use chemix_utils::math::narrow;
use chrono::offset::LocalResult;
use chrono::offset::Local;
use std::sync::MutexGuard;
use std::time;
use chemix_models::struct2array;
use chemix_utils::time::get_current_time;

#[derive(RustcEncodable,Deserialize, Debug,PartialEq,Clone,Serialize)]
pub enum Status {
    #[serde(rename = "matched")]
    Matched,
    #[serde(rename = "launched")]
    Launched,
    #[serde(rename = "confirmed")] // 有效区块确认防分叉回滚
    Confirmed,
}

/***
pub fn generate_trade(
    taker: &str,
    maker_order: &postgresql::UpdateOrder,
    engine_trade: &EngineTrade,
    transaction_id: i32,
) -> Vec<String> {
    unsafe {
        let mut trade = postgresql::TradeInfo {
            id: "".to_string(),
            transaction_id,
            transaction_hash: "".to_string(),
            status: "matched".to_string(),
            market_id: crate::market_id.clone(),
            maker: maker_order.trader_address.clone(),
            taker: taker.to_string(),
            price: engine_trade.price,
            amount: engine_trade.amount,
            taker_side: engine_trade.taker_side.clone(),
            maker_order_id: engine_trade.maker_order_id.clone(),
            taker_order_id: engine_trade.taker_order_id.clone(),
            updated_at: get_current_time(),
            created_at: get_current_time(),
        };
        let data = format!(
            "{}{}{}{}{}{}{}{}{}",
            trade.market_id,
            trade.maker,
            trade.taker,
            trade.price,
            trade.amount,
            trade.taker_side,
            trade.maker_order_id,
            trade.taker_order_id,
            trade.created_at
        );
        let txid = sha256(data);
        trade.id = txid;
        let trade_arr = struct2array(&trade);
        trade_arr
    }
}

 */