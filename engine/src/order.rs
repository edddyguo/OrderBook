use std::collections::HashMap;
use std::ops::{Add, Sub};
use ethers_core::types::{I256, U256};

use serde::Serialize;

use crate::AddBook2;
//use ethers::{prelude::*,types::{U256}};
use serde::Deserialize;
use chemix_chain::chemix::CancelOrderState2;

use chemix_utils::math::narrow;

use chemix_models::order::{BookOrder, get_order};
use chemix_models::order::IdOrIndex::Index;
use chemix_models::trade::{TradeInfo};
use common::types::order::Status as OrderStatus;
use common::types::trade::Status as TradeStatus;
use common::types::order::Side as OrderSide;


#[derive(RustcEncodable, Clone, Serialize)]
pub struct EventOrder {
    pub market_id: String,
    pub side: OrderSide,
    pub price: f64,
    pub amount: f64,
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
    marker_reduced_orders: &mut HashMap<String, U256>,
) -> U256 {
    let u256_zero = U256::from(0i32);
    let book = &mut crate::BOOK.lock().unwrap();
    let mut total_matched_amount = U256::from(0i32);
    'marker_orders: loop {
        match &taker_order.side {
            OrderSide::Buy => {
                if book.sell.is_empty() || taker_order.price < book.sell.first().unwrap().price
                {
                    //此时一定是有吃单剩余
                    info!("______0001__{:?}",orders.asks.get(&taker_order.price));
                    let stat = orders
                        .bids
                        .entry(taker_order.price.clone())
                        .or_insert(I256::from(0i32));
                    *stat += I256::from_raw(taker_order.amount);

                    info!("______0002__{:?}",orders.bids.get(&taker_order.price));

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
                        marker_order.price.clone(),
                        matched_amount.clone(),
                        taker_order.side.clone(),
                        marker_order.id.clone(),
                        taker_order.id.clone(),
                    ));

                    //update asks
                    let stat = orders
                        .asks
                        .entry(marker_order.price.clone())
                        .or_insert(I256::from(0i32));
                    *stat -= I256::from_raw(matched_amount);

                    //get marker_order change value
                    marker_reduced_orders
                        .insert(marker_order.id.clone(), matched_amount);

                    marker_order.amount = marker_order.amount.sub(matched_amount);
                    //todo: 不在去减，用total_matched_amount 判断
                    taker_order.amount = taker_order.amount.sub(matched_amount);
                    total_matched_amount = total_matched_amount.add(matched_amount);
                    if marker_order.amount != u256_zero && taker_order.amount == u256_zero {
                        book.sell[0] = marker_order;
                        break 'marker_orders;
                    } else if marker_order.amount == u256_zero && taker_order.amount != u256_zero {
                        book.sell.remove(0);
                    } else if marker_order.amount == u256_zero && taker_order.amount == u256_zero {
                        book.sell.remove(0);
                        break 'marker_orders;
                    } else {
                        unreachable!()
                    }
                }
            }
            OrderSide::Sell => {
                if book.buy.is_empty() || taker_order.price > book.buy.first().unwrap().price {
                    //此时一定是有吃单剩余
                    info!("______0003__{:?}",orders.asks.get(&taker_order.price));
                    let stat = orders
                        .asks
                        .entry(taker_order.price.clone())
                        .or_insert(I256::from(0i32));
                    *stat += I256::from_raw(taker_order.amount);

                    info!("______0004__{:?}",orders.asks.get(&taker_order.price));


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
                        marker_order.price.clone(),
                        matched_amount,
                        taker_order.side.clone(),
                        marker_order.id.clone(),
                        taker_order.id.clone(),
                    ));

                    //info!("gen new trade {:?}",trades);
                    //update asks
                    let stat = orders
                        .bids
                        .entry(marker_order.price.clone())
                        .or_insert(I256::from(0i32));
                    *stat -= I256::from_raw(matched_amount);

                    //get change marker order
                    marker_reduced_orders
                        .insert(marker_order.id.clone(), matched_amount);

                    marker_order.amount =  marker_order.amount.sub(matched_amount);
                    taker_order.amount = taker_order.amount.sub(matched_amount);
                    total_matched_amount = total_matched_amount.add(matched_amount);
                    if marker_order.amount != u256_zero && taker_order.amount == u256_zero {
                        book.buy[0] = marker_order;
                        break 'marker_orders;
                    } else if marker_order.amount == u256_zero && taker_order.amount != u256_zero {
                        book.buy.remove(0);
                    } else if marker_order.amount == u256_zero && taker_order.amount == u256_zero {
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

pub fn cancel(new_cancel_orders : Vec<CancelOrderState2>) -> Vec<CancelOrderState2>{
    let mut legal_orders = Vec::new();
    for new_cancel_order in new_cancel_orders {
        //todo: 处理异常
        let order = get_order(Index(new_cancel_order.order_index.as_u32())).unwrap();
        match order.status {
            OrderStatus::FullFilled => {
                warn!("Have already matched");
            }
            OrderStatus::PartialFilled => {
                //todo: side 处理
                match order.side {
                    OrderSide::Buy => {
                        crate::BOOK.lock().unwrap().buy.retain(|x| x.id != order.id);
                        legal_orders.push(new_cancel_order);
                    },
                    OrderSide::Sell => {
                        crate::BOOK.lock().unwrap().sell.retain(|x| x.id != order.id);
                        legal_orders.push(new_cancel_order);
                    }
                }
            }
            OrderStatus::Pending => {
                match order.side {
                    OrderSide::Buy => {
                        crate::BOOK.lock().unwrap().buy.retain(|x| x.id != order.id);
                        legal_orders.push(new_cancel_order);

                    },
                    OrderSide::Sell => {
                        crate::BOOK.lock().unwrap().sell.retain(|x| x.id != order.id);
                        legal_orders.push(new_cancel_order);
                    }
                }
            }
            OrderStatus::Canceled => {
                warn!("Have already Canceled");
            }
            OrderStatus::Abandoned => {
                todo!()
            }
        }

    }
    legal_orders
}

pub fn flush() {
    todo!()
}
