#![feature(map_first_last)]
#![deny(unsafe_code)]
//#![deny(warnings)]

pub mod book;
pub mod order;
mod rollback;

use ethers::prelude::*;
use std::collections::{BTreeMap, HashMap};

use chemix_chain::bsc::{get_block, get_current_block};
use chemix_chain::chemix::ChemixContractClient;
use rsmq_async::{Rsmq, RsmqConnection};
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::string::String;

use crate::order::{legal_cancel_orders_filter, legal_new_orders_filter, match_order};
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use std::time;

use tokio::runtime::Runtime;

use clap::{App, Arg};

use chemix_models::order::{
    insert_orders, list_orders, update_orders, OrderFilter, OrderInfoPO, UpdateOrder,
};
use chemix_models::trade::{insert_trades, TradeInfoPO};

use chemix_chain::chemix::storage::{CancelOrderState2, Storage};
use common::utils::math::{u256_to_f64, U256_ZERO};
use common::utils::time::{get_current_time, get_unix_time};

use chemix_models::market::{get_markets, MarketInfoPO};

use crate::book::{
    gen_engine_buy_order, gen_engine_sell_order, Book, BookValue, BuyPriority, SellPriority,
};
use chemix_models::thaws::{insert_thaws, ThawsPO};
use chemix_models::{transactin_begin, transactin_commit};
use common::queue::*;

use common::types::depth::{Depth, RawDepth};
use common::types::order::Status as OrderStatus;
use common::types::order::{Side as OrderSide, Side};
use common::types::trade::AggTrade;
use crate::rollback::get_rollback_point;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

const CONFIRM_HEIGHT: u32 = 2;

lazy_static! {
    static ref MARKET: MarketInfoPO = {
            let matches = App::new("engine")
        .version("1.0")
        .about("Does awesome things")
        .arg(Arg::new("market_id")
            .about("Sets the pem file to use")
            .required(true)
            .index(1))
        .get_matches();
        let market_id : &str = matches.value_of("market_id").unwrap();
        get_markets(market_id).unwrap()
    };

    static ref BOOK: Mutex<Book> = Mutex::new({
        let available_buy_orders = list_orders(OrderFilter::AvailableOrders(MARKET.id.clone(),OrderSide::Buy)).unwrap();
        let available_sell_orders = list_orders(OrderFilter::AvailableOrders(MARKET.id.clone(),OrderSide::Sell)).unwrap();

        //todo: 统一数据结构
        let available_sell = available_sell_orders.iter().map(|x|{
            gen_engine_sell_order(x)
        }).collect::<Vec<(SellPriority,BookValue)>>();

        let available_buy = available_buy_orders.iter().map(|x|{
            gen_engine_buy_order(x)
        }).collect::<Vec<(BuyPriority,BookValue)>>();

        Book {
            buy: BTreeMap::from_iter(available_buy.into_iter()),
            sell: BTreeMap::from_iter(available_sell.into_iter()),
        }
    });


}

fn gen_depth_from_cancel_orders(pending_thaws: Vec<ThawsPO>) -> RawDepth {
    let mut add_depth = RawDepth {
        asks: HashMap::<U256, I256>::new(),
        bids: HashMap::<U256, I256>::new(),
    };

    let mut update_depth = |x: ThawsPO| {
        let amount = I256::try_from(x.amount).unwrap();
        match x.side {
            Side::Buy => {
                let new_bids = add_depth.bids.entry(x.price).or_insert(I256::from(0i32));
                *new_bids -= amount;
            }
            Side::Sell => {
                let new_asks = add_depth.asks.entry(x.price).or_insert(I256::from(0i32));
                *new_asks -= amount;
            }
        }
    };

    for pending_thaw in pending_thaws {
        update_depth(pending_thaw);
    }
    add_depth
}

fn gen_depth_from_raw(add_depth: RawDepth) -> Depth {
    let base_token_decimal = crate::MARKET.base_contract_decimal;
    let quote_token_decimal = crate::MARKET.quote_contract_decimal;
    info!("test_decimal {:?}", *crate::MARKET);
    let asks2 = add_depth
        .asks
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), quote_token_decimal);
            let user_volume = if y < &I256::from(0u32) {
                u256_to_f64(y.abs().into_raw(), base_token_decimal) * -1.0f64
            } else {
                u256_to_f64(y.abs().into_raw(), base_token_decimal)
            };
            (user_price, user_volume)
        })
        .filter(|(p, v)| p != &0.0 && v != &0.0)
        .collect::<Vec<(f64, f64)>>();

    let bids2 = add_depth
        .bids
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), quote_token_decimal);
            let user_volume = if y < &I256::from(0u32) {
                u256_to_f64(y.abs().into_raw(), base_token_decimal) * -1.0f64
            } else {
                u256_to_f64(y.abs().into_raw(), base_token_decimal)
            };
            (user_price, user_volume)
        })
        .filter(|(p, v)| p != &0.0 && v != &0.0)
        .collect::<Vec<(f64, f64)>>();

    Depth {
        asks: asks2,
        bids: bids2,
    }
}

fn gen_agg_trade_from_raw(trades: Vec<TradeInfoPO>) -> Vec<AggTrade> {
    trades
        .into_iter()
        .map(|x| AggTrade {
            id: x.id,
            taker: x.taker.clone(),
            maker: x.maker.clone(),
            price: u256_to_f64(x.price, crate::MARKET.quote_contract_decimal),
            amount: u256_to_f64(x.amount, crate::MARKET.base_contract_decimal),
            height: -1,
            taker_side: x.taker_side,
            updated_at: get_unix_time(),
        })
        .filter(|x| x.price != 0.0 && x.amount != 0.0)
        .collect::<Vec<AggTrade>>()
}

async fn send_depth_message(depth: Depth, arc_queue: &Arc<RwLock<Rsmq>>) {
    let mut market_depth = HashMap::new();
    market_depth.insert(crate::MARKET.id.clone(), depth);
    let json_str = serde_json::to_string(&market_depth).unwrap();
    arc_queue
        .write()
        .unwrap()
        .send_message(&QueueType::Depth.to_string(), json_str, None)
        .await
        .expect("failed to send message");
}

async fn send_agg_trade_message(agg_trade: Vec<AggTrade>, arc_queue: &Arc<RwLock<Rsmq>>) {
    let mut market_agg_trade = HashMap::new();
    market_agg_trade.insert(crate::MARKET.id.clone(), agg_trade);
    let json_str = serde_json::to_string(&market_agg_trade).unwrap();
    info!("trade_push {}", json_str);
    arc_queue
        .write()
        .unwrap()
        .send_message(QueueType::Trade.to_string().as_str(), json_str, None)
        .await
        .expect("failed to send message");
}

async fn listen_blocks(queue: Rsmq) -> anyhow::Result<()> {
    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let arc_queue= Arc::new(RwLock::new(queue));
    //不考虑安全性,随便写个私钥
    let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
    let chemix_storage_client = ChemixContractClient::<Storage>::new(pri_key);
    let chemix_storage_client = Arc::new(RwLock::new(chemix_storage_client));
    info!("__0001");
    rayon::scope(|s| {
        //监听合约事件（新建订单和取消订单），将其发送到相应处理模块
        let arc_queue_engine = arc_queue.clone();
        let arc_queue_chain_listener = arc_queue.clone();
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let last_order = list_orders(OrderFilter::GetLastOne).unwrap();
                let mut last_process_height = if last_order.is_empty() {
                    get_current_block().await
                } else {
                    last_order[0].block_height
                };

                loop {
                    //如果有订单分叉，则等待回滚完毕，并且从回滚点重新检查交易
                    if let Some(rollback_height) = get_rollback_point(&arc_queue_chain_listener).await{
                        last_process_height = rollback_height;
                    }
                    let current_height = get_current_block().await;
                    assert!(current_height >= last_process_height);
                    if current_height - last_process_height <= CONFIRM_HEIGHT {
                        info!("current chain height {},wait for new block", current_height);
                        tokio::time::sleep(time::Duration::from_millis(1000)).await;
                    } else {
                        debug!("current_book {:#?}", crate::BOOK.lock().unwrap());
                        //规避RPC阻塞等网络问题导致的没有及时获取到最新块高，以及系统重启时期对离线期间区块的处理
                        //绝大多数情况last_process_height + 1 等于current_height - CONFIRM_HEIGHT
                        for height in last_process_height + 1..=current_height - CONFIRM_HEIGHT
                        {
                            let block_hash = get_block(BlockId::from(height as u64))
                                .await
                                .unwrap()
                                .unwrap()
                                .hash
                                .unwrap();
                            info!("deal with block height {}", height);
                            //区块中的取消订单
                            let new_cancel_orders = chemix_storage_client
                                .clone()
                                .write()
                                .unwrap()
                                .filter_new_cancel_order_created_event(
                                    block_hash,
                                    crate::MARKET.base_token_address.clone(),
                                    crate::MARKET.quote_token_address.clone(),
                                )
                                .await
                                .unwrap();
                            info!("new_cancel_orders_event {:?}", new_cancel_orders);
                            let legal_orders =
                                legal_cancel_orders_filter(new_cancel_orders.clone());

                            //区块中新创建的订单
                            let new_orders = chemix_storage_client
                                .clone()
                                .write()
                                .unwrap()
                                .filter_new_order_event(
                                    block_hash,
                                    crate::MARKET.base_token_address.clone(),
                                    crate::MARKET.quote_token_address.clone(),
                                )
                                .await
                                .unwrap();
                            info!("new_orders_event {:?} at height {}", new_orders, height);
                            let db_new_orders = legal_new_orders_filter(new_orders, height);

                            //将合法的事件信息推送给撮合模块处理
                            event_sender
                                .send((legal_orders, db_new_orders))
                                .expect("failed to send orders");
                        }
                    }
                    last_process_height = current_height - CONFIRM_HEIGHT;
                    info!(
                        "test1:: last_process_height {}, current_height {}",
                        last_process_height, current_height
                    );
                }
            });
        });

        s.spawn(move |_| {
            loop {
                let now = get_current_time();
                let (legal_orders, mut orders): (Vec<CancelOrderState2>, Vec<OrderInfoPO>) =
                    event_receiver.recv().expect("failed to recv book order");

                let mut add_depth = RawDepth {
                    asks: HashMap::<U256, I256>::new(),
                    bids: HashMap::<U256, I256>::new(),
                };

                //单区块内的所有落表，作为一个事务处理
                transactin_begin();
                //处理取消的订单
                if legal_orders.is_empty() {
                    info!("Not found legal_cancel orders");
                } else {
                    let mut pending_thaws = Vec::new();
                    let mut pre_cancle_orders = Vec::new();
                    for cancel_order in legal_orders {
                        let orders = list_orders(OrderFilter::ByIndex(
                            cancel_order.order_index.as_u32(),
                        ))
                        .unwrap();
                        let order = orders.first().unwrap();
                        let update_info = UpdateOrder {
                            id: order.id.clone(),
                            status: OrderStatus::Canceled,
                            available_amount: U256_ZERO,
                            matched_amount: order.matched_amount,
                            canceled_amount: order.available_amount,
                            updated_at: &now,
                        };
                        pre_cancle_orders.push(update_info);
                        pending_thaws.push(ThawsPO::new(
                            &order.id,
                            &order.account,
                            &order.market_id,
                            order.available_amount,
                            order.price,
                            order.side.clone(),
                        ));
                    }
                    update_orders(&pre_cancle_orders);
                    insert_thaws(&pending_thaws);
                    add_depth = gen_depth_from_cancel_orders(pending_thaws);
                }

                //处理新来的订单
                let mut db_trades = Vec::<TradeInfoPO>::new();
                if orders.is_empty() {
                    info!("Not found legal created orders");
                } else {
                    info!(
                        "[listen_blocks: receive] New order Event {:?},base token {:?}",
                        orders[0].id, orders[0].side
                    );
                    //market_orders的移除或者减少
                    let mut db_marker_orders_reduce = HashMap::<String, U256>::new();
                    for (index, db_order) in orders.iter_mut().enumerate() {
                        match_order(
                            db_order,
                            &mut db_trades,
                            &mut add_depth,
                            &mut db_marker_orders_reduce,
                        );

                        info!("index {},taker amount {}", index, db_order.amount);
                        db_order.status = if db_order.available_amount == U256_ZERO {
                            OrderStatus::FullFilled
                        } else if db_order.available_amount != U256_ZERO
                            && db_order.available_amount < db_order.amount
                        {
                            OrderStatus::PartialFilled
                        } else if db_order.available_amount == db_order.amount {
                            OrderStatus::Pending
                        } else {
                            unreachable!()
                        };
                        db_order.matched_amount = db_order.amount - db_order.available_amount;
                        info!(
                            "finished match_order index {},and status {:?},status_str={},",
                            index,
                            db_order.status,
                            db_order.status.as_str()
                        );
                    }
                    info!("Generate trades {:?},and flush those to db", db_trades);

                    insert_orders(&orders);

                    //update marker orders
                    info!("db_marker_orders_reduce {:?}", db_marker_orders_reduce);
                    let mut pre_update_orders = Vec::new();
                    for orders in db_marker_orders_reduce {
                        let market_orders = list_orders(OrderFilter::ById(&orders.0)).unwrap();
                        let marker_order_ori = market_orders.first().unwrap();
                        let new_matched_amount = marker_order_ori.matched_amount + orders.1;
                        info!(
                            "marker_order_ori {};available_amount={},reduce_amount={}",
                            marker_order_ori.id, marker_order_ori.available_amount, orders.1
                        );
                        let new_available_amount = marker_order_ori.available_amount - orders.1;

                        let new_status = if new_available_amount == U256_ZERO {
                            OrderStatus::FullFilled
                        } else {
                            OrderStatus::PartialFilled
                        };

                        let update_info = UpdateOrder {
                            id: orders.0,
                            status: new_status,
                            available_amount: new_available_amount,
                            canceled_amount: marker_order_ori.canceled_amount,
                            matched_amount: new_matched_amount,
                            updated_at: &now,
                        };
                        pre_update_orders.push(update_info);
                    }
                    if !db_trades.is_empty() {
                        insert_trades(&mut db_trades);
                        update_orders(&pre_update_orders);
                    }
                }

                transactin_commit();
                let rt = Runtime::new().unwrap();
                let arc_queue_engine = arc_queue_engine.clone();
                rt.block_on(async move {
                    if add_depth != Default::default() {
                        let depth = gen_depth_from_raw(add_depth);
                        send_depth_message(depth, &arc_queue_engine).await;
                    }
                    let trades = gen_agg_trade_from_raw(db_trades);
                    if !trades.is_empty() {
                        send_agg_trade_message(trades, &arc_queue_engine).await;
                    }
                });
            }
        });
    });

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let queue = Queue::regist(vec![QueueType::Depth, QueueType::Trade, QueueType::Thaws,QueueType::Chain]).await;
    info!("initial book {:#?}", crate::BOOK.lock().unwrap());
    listen_blocks(queue).await
}
