use ethers_core::types::{I256, U256};
use std::collections::HashMap;
use std::ops::{Add, Sub};

use serde::Serialize;

use crate::{BookValue, BuyPriority};
//use ethers::{prelude::*,types::{U256}};

use chemix_chain::chemix::storage::{CancelOrderState2, ChainNewOrder};

use chemix_models::order::{list_orders, OrderFilter, OrderInfoPO};
use chemix_models::trade::TradeInfoPO;
use common::types::depth::{Depth, RawDepth};
use common::types::order::{Side, Status as OrderStatus};

use common::types::order::Side as OrderSide;
use common::utils::math::{u256_to_f64, U256_ZERO};

use crate::book::SellPriority;

#[derive(Clone, Serialize)]
pub struct EventOrder {
    pub market_id: String,
    pub side: OrderSide,
    pub price: f64,
    pub amount: f64,
}

pub fn match_order(
    mut taker_order: &mut OrderInfoPO,
    trades: &mut Vec<TradeInfoPO>,
    raw_depth: &mut RawDepth,
    marker_reduced_orders: &mut HashMap<String, U256>,
) {
    let book = &mut crate::BOOK.lock().unwrap();
    let mut total_matched_amount = U256_ZERO;
    'marker_orders: loop {
        info!("test_001");
        match &taker_order.side {
            OrderSide::Buy => {
                //不能吃单的直接挂单
                if book.sell.is_empty()
                    || taker_order.price < book.sell.first_key_value().unwrap().0.price
                {
                    info!("test_002");
                    //insert this order by compare price and created_at
                    book.buy.insert(
                        BuyPriority {
                            price: taker_order.price,
                            order_index: taker_order.index,
                        },
                        BookValue {
                            id: taker_order.id.clone(),
                            account: taker_order.account.clone(),
                            side: taker_order.side.clone(),
                            amount: taker_order.available_amount,
                        },
                    );

                    //剩余的订单使深度增加
                    let stat = raw_depth
                        .bids
                        .entry(taker_order.price)
                        .or_insert(I256::from(0i32));
                    *stat += I256::from_raw(taker_order.available_amount);
                    /***
                    book.buy
                        .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
                    book.buy.reverse();

                     */
                    break 'marker_orders;
                } else {
                    let mut marker_order = book.sell.pop_first().unwrap();
                    let matched_amount =
                        std::cmp::min(taker_order.available_amount, marker_order.1.amount);

                    trades.push(TradeInfoPO::new(
                        &crate::MARKET.id,
                        &taker_order.account,
                        &marker_order.1.account,
                        marker_order.0.price,
                        matched_amount,
                        taker_order.side.clone(),
                        &marker_order.1.id,
                        &taker_order.id,
                    ));

                    //吃掉的订单使卖单深度减少
                    let stat = raw_depth
                        .asks
                        .entry(marker_order.0.price)
                        .or_insert(I256::from(0i32));
                    *stat -= I256::from_raw(matched_amount);

                    //get marker_order change value
                    marker_reduced_orders.insert(marker_order.1.id.clone(), matched_amount);
                    marker_order.1.amount = marker_order.1.amount.sub(matched_amount);
                    //todo: 不在去减，用total_matched_amount 判断
                    taker_order.available_amount =
                        taker_order.available_amount.sub(matched_amount);
                    total_matched_amount = total_matched_amount.add(matched_amount);
                    if marker_order.1.amount != U256_ZERO
                        && taker_order.available_amount == U256_ZERO
                    {
                        //book.sell[0] = marker_order;
                        book.sell.insert(marker_order.0, marker_order.1);
                        break 'marker_orders;
                    } else if marker_order.1.amount == U256_ZERO
                        && taker_order.available_amount != U256_ZERO
                    {
                        continue;
                    } else if marker_order.1.amount == U256_ZERO
                        && taker_order.available_amount == U256_ZERO
                    {
                        break 'marker_orders;
                    } else {
                        unreachable!()
                    }
                }
            }
            OrderSide::Sell => {
                if book.buy.is_empty()
                    || taker_order.price > book.buy.first_key_value().unwrap().0.price
                {
                    //insert this order by compare price and created_at
                    book.sell.insert(
                        SellPriority {
                            price: taker_order.price,
                            order_index: taker_order.index,
                        },
                        BookValue {
                            id: taker_order.id.clone(),
                            account: taker_order.account.clone(),
                            side: taker_order.side.clone(),
                            amount: taker_order.available_amount,
                        },
                    );

                    let stat = raw_depth
                        .asks
                        .entry(taker_order.price)
                        .or_insert(I256::from(0i32));
                    *stat += I256::from_raw(taker_order.available_amount);
                    break 'marker_orders;
                } else {
                    let mut marker_order = book.buy.pop_first().unwrap();
                    let matched_amount =
                        std::cmp::min(taker_order.available_amount, marker_order.1.amount);

                    trades.push(TradeInfoPO::new(
                        &crate::MARKET.id,
                        &taker_order.account,
                        &marker_order.1.account,
                        marker_order.0.price,
                        matched_amount,
                        taker_order.side.clone(),
                        &marker_order.1.id,
                        &taker_order.id,
                    ));

                    let stat = raw_depth
                        .bids
                        .entry(marker_order.0.price)
                        .or_insert(I256::from(0i32));
                    *stat -= I256::from_raw(matched_amount);

                    //get change marker order
                    marker_reduced_orders.insert(marker_order.1.id.clone(), matched_amount);

                    marker_order.1.amount = marker_order.1.amount.sub(matched_amount);
                    taker_order.available_amount =
                        taker_order.available_amount.sub(matched_amount);
                    total_matched_amount = total_matched_amount.add(matched_amount);
                    if marker_order.1.amount != U256_ZERO
                        && taker_order.available_amount == U256_ZERO
                    {
                        book.buy.insert(marker_order.0, marker_order.1);
                        break 'marker_orders;
                    } else if marker_order.1.amount == U256_ZERO
                        && taker_order.available_amount != U256_ZERO
                    {
                        continue;
                    } else if marker_order.1.amount == U256_ZERO
                        && taker_order.available_amount == U256_ZERO
                    {
                        break 'marker_orders;
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }
}

pub fn legal_cancel_orders_filter(
    new_cancel_orders: Vec<CancelOrderState2>,
) -> Vec<CancelOrderState2> {
    let mut legal_orders = Vec::<CancelOrderState2>::new();
    for new_cancel_order in new_cancel_orders {
        let orders =
            list_orders(OrderFilter::ByIndex(new_cancel_order.order_index.as_u32())).unwrap();
        if orders.is_empty() {
            warn!("Order index {} not found", new_cancel_order.order_index);
            return legal_orders;
        }
        let order = orders[0].clone();
        //防止一个区块内的多次取消的情况，多次取消以最后一次为有效
        legal_orders.retain(|x| x.order_index.as_u32() != order.index);
        match order.status {
            OrderStatus::FullFilled => {
                warn!("Have already matched");
            }
            OrderStatus::PartialFilled => match order.side {
                OrderSide::Buy => {
                    crate::BOOK
                        .lock()
                        .unwrap()
                        .buy
                        .retain(|_, v| v.id != order.id);
                    legal_orders.push(new_cancel_order);
                }
                OrderSide::Sell => {
                    crate::BOOK
                        .lock()
                        .unwrap()
                        .sell
                        .retain(|_, v| v.id != order.id);
                    legal_orders.push(new_cancel_order);
                }
            },
            OrderStatus::Pending => match order.side {
                OrderSide::Buy => {
                    crate::BOOK
                        .lock()
                        .unwrap()
                        .buy
                        .retain(|_, v| v.id != order.id);
                    legal_orders.push(new_cancel_order);
                }
                OrderSide::Sell => {
                    crate::BOOK
                        .lock()
                        .unwrap()
                        .sell
                        .retain(|_, v| v.id != order.id);
                    legal_orders.push(new_cancel_order);
                }
            },
            OrderStatus::Canceled => {
                warn!("Have already Canceled");
            }
        }
    }
    legal_orders
}

pub fn legal_new_orders_filter(
    raw_orders: Vec<ChainNewOrder>,
    height: u32,
) -> Vec<OrderInfoPO> {
    let mut db_new_orders = Vec::new();
    for order in raw_orders {
        let base_decimal = crate::MARKET.base_contract_decimal as u32;

        let raw_amount = if base_decimal > order.num_power {
            order.amount * u256_power!(10u32,(base_decimal - order.num_power))
        } else {
            order.amount / u256_power!(10u32,(order.num_power - base_decimal))
        };
        info!(
            "amount_ori {},order.num_power {},amount_cur {}",
            order.amount, order.num_power, raw_amount
        );
        //如果num_power过大，则raw_amount为零无效
        if raw_amount != U256_ZERO {
            db_new_orders.push(OrderInfoPO::new(
                order.id,
                order.index,
                order.transaction_hash,
                height,
                order.hash_data,
                crate::MARKET.id.to_string(),
                order.account,
                order.side,
                order.price,
                raw_amount,
            ));
        }
    }
    db_new_orders
}

//根据未成交的订单生成深度数据
pub fn gen_depth_from_order(orders: Vec<OrderInfoPO>) -> HashMap<String, Depth> {
    let mut raw_depth = RawDepth {
        asks: HashMap::new(),
        bids: HashMap::new(),
    };

    for order in orders {
        match order.side {
            Side::Buy => {
                let stat = raw_depth
                    .bids
                    .entry(order.price)
                    .or_insert(I256::from(0i32));
                *stat += I256::from_raw(order.amount);
            }
            Side::Sell => {
                let stat = raw_depth
                    .asks
                    .entry(order.price)
                    .or_insert(I256::from(0i32));
                *stat += I256::from_raw(order.amount);
            }
        }
    }

    let base_decimal = crate::MARKET.base_contract_decimal as u32;
    let quote_decimal = crate::MARKET.quote_contract_decimal as u32;

    let asks = raw_depth
        .asks
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), quote_decimal);
            let user_volume = if y < &I256::from(0u32) {
                u256_to_f64(y.abs().into_raw(), base_decimal) * -1.0f64
            } else {
                u256_to_f64(y.abs().into_raw(), base_decimal)
            };

            (user_price, user_volume)
        })
        .filter(|(p, v)| p != &0.0 && v != &0.0)
        .collect::<Vec<(f64, f64)>>();

    let bids = raw_depth
        .bids
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), quote_decimal);
            let user_volume = if y < &I256::from(0u32) {
                u256_to_f64(y.abs().into_raw(), base_decimal) * -1.0f64
            } else {
                u256_to_f64(y.abs().into_raw(), base_decimal)
            };
            (user_price, user_volume)
        })
        .filter(|(p, v)| p != &0.0 && v != &0.0)
        .collect::<Vec<(f64, f64)>>();

    let mut market_add_depth = HashMap::new();
    market_add_depth.insert(crate::MARKET.id.clone(), Depth { asks, bids });
    market_add_depth
}
