use rustc_serialize::json;
use serde::Serialize;
use std::ops::Deref;
use crate::{AddBook, LastTrade, EngineBook};
//use ethers::{prelude::*,types::{U256}};
use serde::Deserialize;
use utils::algorithm::sha256;
use utils::math::narrow;
use chrono::offset::LocalResult;
use chrono::offset::Local;
use std::sync::MutexGuard;



#[derive(RustcEncodable, Clone, Serialize)]
pub struct EventOrder {
    pub market_id: String,
    pub side: String,
    pub price: f64,
    pub amount: f64,
}

#[derive(Clone, Serialize,Debug)]
pub struct BookOrder {
    pub id: String,
    pub side: String,
    pub price: u64,
    pub amount: u64,
    pub created_at: u64,
}
#[derive(Deserialize, Debug, Default, Clone)]
pub struct EngineOrder {
    pub id: String,
    pub trader_address: String,
    pub status: String,
    pub total_amount: u64,
    pub available_amount: u64,
    pub updated_at: u64,
}

/**
#[derive(Clone, Serialize,Debug)]
pub struct EngineOrder {
    pub id: String,
    pub side: String,
    pub price: u64,
    pub amount: u64,
    pub created_at: u64,
}
*/
/**
pub fn match_order(mut order: BookOrder) -> (AddBook,Vec<LastTrade>) {
    //fix: rc
    let total_amount = order.amount;
    let mut book : MutexGuard<EngineBook> = crate::BOOK.lock().unwrap();
    let mut sum_matched: u64 = 0;
    let mut matched_amount: u64 = 0;
    //book.buy.push()
    let mut opponents_book = if order.side == "buy" {
        book.sell.clone()
    }else {
        book.buy.clone()
    };
    let can_match = || -> bool {
        if opponents_book.is_empty() {
           return false
        }
        let gap_price = order.price as i64 - opponents_book[0].price as i64;
        if (order.side == "buy" && gap_price < 0) || (order.side == "sell" && gap_price > 0){
            false
        }else if  (order.side == "buy" && gap_price >= 0) || (order.side == "sell" && gap_price <= 0) {
            true
        }else {
            unreachable!()
        }
    };

    //fixme: 先在match_order进行落盘，后期挪到其他线程
    let mut trades = Vec::<LastTrade>::new();
    let mut update_book = AddBook {
        asks: vec![],
        bids: vec![],
    };
    'batch_orders : loop{
        if !can_match() {
            //todo: insert this order
            break;
        }

        let matched_amount = std::cmp::min(order.amount.clone(),opponents_book[0].amount);

        //fix: 编译有错
        order.amount -= opponents_book[1].amount;
        if order.amount == 0 {
            break;
        }

        let now = Local::now().timestamp_millis() as u64;
        let mut match_trade = LastTrade{
            id: "".to_string(),
            price: narrow(opponents_book[1].price),
            amount: narrow(matched_amount),
            taker_side: order.side.clone(),
            updated_at: now,
        };

        match_trade.id = sha256(serde_json::to_string(&match_trade).unwrap());
        opponents_book.push(order.clone());
        trades.push(match_trade);
    }
    (update_book, trades)
}
*/
pub fn cancel(){
    todo!()
}

pub fn flush(){
    todo!()
}