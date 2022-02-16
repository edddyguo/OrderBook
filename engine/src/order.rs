use std::collections::HashMap;

use serde::Serialize;

use crate::AddBook2;
//use ethers::{prelude::*,types::{U256}};
use serde::Deserialize;

use chemix_utils::math::narrow;

use chemix_models::order::Side;
use chemix_models::trade::TradeInfo;

//  status text , --"full_filled","partial_filled","pending"
#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
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

#[derive(Clone, Serialize, Debug)]
pub struct BookOrder {
    pub id: String,
    pub account: String,
    pub side: Side,
    pub price: u64,
    pub amount: u64,
    pub created_at: u64,
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

pub fn match_order(
    mut taker_order: BookOrder,
    trades: &mut Vec<TradeInfo>,
    orders: &mut AddBook2,
    marker_reduced_orders: &mut HashMap<String, f64>,
) -> u64 {
    let book = &mut crate::BOOK.lock().unwrap();
    let mut total_matched_amount: u64 = 0;
    'marker_orders: loop {
        match &taker_order.side {
            Side::Buy => {
                if book.sell.is_empty() || taker_order.price < book.sell.first().unwrap().price
                {
                    //此时一定是有吃单剩余
                    let stat = orders
                        .bids
                        .entry(taker_order.price.clone())
                        .or_insert(taker_order.amount);
                    *stat += taker_order.amount;

                    //insert this order by compare price and created_at
                    //fixme:tmpcode,优化，还有时间排序的问题
                    book.buy.push(taker_order);
                    book.buy
                        .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                    book.buy.reverse();
                    break 'marker_orders;
                } else {
                    let mut marker_order = book.sell[0].clone();
                    let matched_amount = std::cmp::min(taker_order.amount, marker_order.amount);

                    trades.push(TradeInfo::new(
                        taker_order.account.clone(),
                        marker_order.account.clone(),
                        narrow(marker_order.price.clone()),
                        narrow(matched_amount),
                        taker_order.side.clone(),
                        marker_order.id.clone(),
                        taker_order.id.clone(),
                    ));

                    //update asks
                    let stat = orders
                        .asks
                        .entry(marker_order.price.clone())
                        .or_insert(matched_amount);
                    *stat += matched_amount;

                    //get marker_order change value
                    marker_reduced_orders
                        .insert(marker_order.id.clone(), narrow(matched_amount));

                    marker_order.amount -= matched_amount;
                    //todo: 不在去减，用total_matched_amount 判断
                    taker_order.amount -= matched_amount;
                    total_matched_amount += matched_amount;
                    if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.sell[0] = marker_order;
                        break 'marker_orders;
                    } else if marker_order.amount == 0 && taker_order.amount != 0 {
                        book.sell.remove(0);
                    } else if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.sell.remove(0);
                        break 'marker_orders;
                    } else {
                        unreachable!()
                    }
                }
            }
            Side::Sell => {
                if book.buy.is_empty() || taker_order.price > book.buy.first().unwrap().price {
                    //此时一定是有吃单剩余
                    let stat = orders
                        .asks
                        .entry(taker_order.price.clone())
                        .or_insert(taker_order.amount);
                    *stat += taker_order.amount;

                    //insert this order by compare price and created_at
                    //fixme:tmpcode,优化，还有时间的问题
                    book.sell.push(taker_order);
                    book.sell
                        .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                    break 'marker_orders;
                } else {
                    let mut marker_order = book.buy[0].clone();
                    let matched_amount = std::cmp::min(taker_order.amount, marker_order.amount);

                    trades.push(TradeInfo::new(
                        taker_order.account.clone(),
                        marker_order.account.clone(),
                        narrow(marker_order.price.clone()),
                        narrow(matched_amount),
                        taker_order.side.clone(),
                        marker_order.id.clone(),
                        taker_order.id.clone(),
                    ));

                    //info!("gen new trade {:?}",trades);
                    //update asks
                    let stat = orders
                        .bids
                        .entry(marker_order.price.clone())
                        .or_insert(matched_amount);
                    *stat += matched_amount;

                    //get change marker order
                    marker_reduced_orders
                        .insert(marker_order.id.clone(), narrow(matched_amount));

                    marker_order.amount -= matched_amount;
                    taker_order.amount -= matched_amount;
                    total_matched_amount += matched_amount;
                    if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.buy[0] = marker_order;
                        break 'marker_orders;
                    } else if marker_order.amount == 0 && taker_order.amount != 0 {
                        book.buy.remove(0);
                    } else if marker_order.amount != 0 && taker_order.amount == 0 {
                        book.buy.remove(0);
                        break 'marker_orders;
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }

    //todo: update orders and trades in psql

    //drop(book);
    //info!("current book = {:?}",crate::BOOK.lock().unwrap());
    info!("current book = {:?}", book);
    //match_trade.id = sha256(serde_json::to_string(&match_trade).unwrap());

    //(add_depth, trades)
    total_matched_amount
}

pub fn cancel() {
    todo!()
}

pub fn flush() {
    todo!()
}
