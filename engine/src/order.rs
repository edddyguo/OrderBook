use std::borrow::BorrowMut;
use std::collections::HashMap;
use rustc_serialize::json;
use serde::Serialize;
use std::ops::{Deref, Index, Sub};
use crate::{AddBook, LastTrade, EngineBook, AddBook2, LastTrade2};
//use ethers::{prelude::*,types::{U256}};
use serde::Deserialize;
use utils::algorithm::sha256;
use utils::math::narrow;
use chrono::offset::LocalResult;
use chrono::offset::Local;
use std::sync::MutexGuard;
use std::time;

#[derive(RustcEncodable,Deserialize, Debug,PartialEq,Clone,Serialize)]
pub enum Side {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}
//  status text , --"full_filled","partial_filled","pending"
#[derive(RustcEncodable,Deserialize, Debug,PartialEq,Clone,Serialize)]
pub enum Status {
    #[serde(rename = "full_filled")]
    FullFilled,
    #[serde(rename = "partial_filled")]
    PartialFilled,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "canceled")]
    Canceled,
}

#[derive(RustcEncodable, Clone, Serialize)]
pub struct EventOrder {
    pub market_id: String,
    pub side: Side,
    pub price: f64,
    pub amount: f64,
}

#[derive(Clone, Serialize,Debug)]
pub struct BookOrder {
    pub id: String,
    pub account: String,
    pub side: Side,
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

/***
#[derive(Clone, Serialize,Debug)]
pub struct EngineOrder {
    pub id: String,
    pub side: String,
    pub price: u64,
    pub amount: u64,
    pub created_at: u64,
}
*/


pub fn match_order(mut taker_order: BookOrder, agg_trades: &mut Vec<LastTrade2>, add_depth: &mut AddBook2) -> u64{
    let mut book  = & mut crate::BOOK.lock().unwrap();
    let mut total_matched_amount: u64 = 0;
    info!(" _0001");
    'marker_orders : loop {
        info!(" _0002");
        match &taker_order.side {
            Side::Buy => {
                info!(" _0003");
                if book.sell.is_empty() || taker_order.price < book.sell.first().unwrap().price {
                    //此时一定是有吃单剩余
                    let stat = add_depth.bids.entry(taker_order.price.clone()).or_insert(taker_order.amount);
                    *stat += taker_order.amount;

                    //insert this order by compare price and created_at
                    //fixme:tmpcode,优化，还有时间排序的问题
                    book.buy.push(taker_order);
                    book.buy.sort_by(|a,b| {
                        a.price.partial_cmp(&b.price).unwrap()
                    });
                    book.buy.reverse();
                    info!(" _0004");
                    break 'marker_orders;
                }else {
                    info!(" _0005");
                    let mut marker_order = book.sell[0].clone();
                    let matched_amount = std::cmp::min(taker_order.amount,marker_order.amount);
                    agg_trades.push(LastTrade2{
                        price: marker_order.price.clone(),
                        amount: matched_amount,
                        taker_side: taker_order.side.clone(),
                    });

                    //update asks
                    let stat = add_depth.asks.entry(marker_order.price.clone()).or_insert(matched_amount);
                    *stat += matched_amount;

                    marker_order.amount -= matched_amount;
                    //todo: 不在去减，用total_matched_amount 判断
                    taker_order.amount -= matched_amount;
                    total_matched_amount += matched_amount;
                    if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.sell[0] = marker_order;
                        break 'marker_orders;
                    }else if  marker_order.amount == 0 && taker_order.amount != 0 {
                        book.sell.pop();
                    }else if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.sell.pop();
                        break 'marker_orders;
                    }else {
                        unreachable!()
                    }
                }
            }
            Side::Sell => {
                if book.buy.is_empty() || taker_order.price > book.buy.first().unwrap().price {
                    //此时一定是有吃单剩余
                    let stat = add_depth.asks.entry(taker_order.price.clone()).or_insert(taker_order.amount);
                    *stat += taker_order.amount;

                    //insert this order by compare price and created_at
                    //fixme:tmpcode,优化，还有时间的问题
                    book.sell.push(taker_order);
                    book.sell.sort_by(|a,b| {
                        a.price.partial_cmp(&b.price).unwrap()
                    });
                    book.sell.reverse();
                    info!(" _00014");
                    break 'marker_orders;
                }else {
                    info!(" _00015");
                    let mut marker_order = book.buy[0].clone();
                    let matched_amount = std::cmp::min(taker_order.amount,marker_order.amount);
                    agg_trades.push(LastTrade2{
                        price: marker_order.price.clone(),
                        amount: matched_amount,
                        taker_side: taker_order.side.clone(),
                    });

                    //info!("gen new trade {:?}",trades);
                    //update asks
                    let stat = add_depth.bids.entry(marker_order.price.clone()).or_insert(matched_amount);
                    *stat += matched_amount;


                    marker_order.amount -= matched_amount;
                    taker_order.amount -= matched_amount;
                    total_matched_amount += matched_amount;
                    if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.buy[0] = marker_order;
                        break 'marker_orders;
                    }else if  marker_order.amount == 0 && taker_order.amount != 0 {
                        book.buy.pop();
                    }else if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.buy.pop();
                        break 'marker_orders;
                    }else {
                        unreachable!()
                    }

                }
            }
        }
    };


    //todo: update orders and trades in psql

    //drop(book);
    //info!("current book = {:?}",crate::BOOK.lock().unwrap());
    info!("current book = {:?}",book);
    //match_trade.id = sha256(serde_json::to_string(&match_trade).unwrap());

    //(add_depth, trades)
    total_matched_amount
}

pub fn cancel(){
    todo!()
}

pub fn flush(){
    todo!()
}