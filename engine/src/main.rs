pub mod order;

use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;

//use ethers::providers::Ws;

use chemix_chain::chemix::ChemixContractClient;
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use rsmq_async::{Rsmq, RsmqConnection};

use chemix_chain::bsc::{get_block, get_current_block};
use std::string::String;

use serde::Serialize;
use std::convert::TryFrom;

use ethers::types::Address;
use std::fmt::Debug;
use std::ops::Sub;

use std::str::FromStr;

use crate::order::{cancel, match_order};
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use std::time;

use tokio::runtime::Runtime;

use clap::{App, Arg};

use chemix_models::order::{
    insert_order, list_orders, update_order, BookOrder, OrderFilter, OrderInfo, UpdateOrder,
};
use chemix_models::trade::{insert_trades, TradeInfo};

use chemix_chain::chemix::storage::Storage;
use common::utils::math::{u256_to_f64, U256_ZERO};
use common::utils::time::get_current_time;
use common::utils::time::time2unix;

use chemix_models::market::{get_markets, MarketInfo};

use chemix_models::thaws::{insert_thaws, Thaws};
use common::queue::*;
use common::types::order::Status as OrderStatus;
use common::types::order::{Side as OrderSide, Side};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

static BaseTokenDecimal: u32 = 18;
static QuoteTokenDecimal: u32 = 15;

const CONFIRM_HEIGHT: u32 = 2;

#[derive(Clone, Serialize, Debug)]
struct EngineBook {
    pub buy: Vec<BookOrder>,
    pub sell: Vec<BookOrder>,
}

#[derive(Clone, Serialize, Debug)]
pub struct EnigneSettleValues {
    pub incomeQuoteToken: I256,
    pub incomeBaseToken: I256,
}

lazy_static! {
    static ref MARKET: MarketInfo = {
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

    static ref BOOK: Mutex<EngineBook> = Mutex::new({
        let available_buy = list_orders(OrderFilter::AvailableOrders(MARKET.id.clone(),OrderSide::Buy)).unwrap();
        let available_sell = list_orders(OrderFilter::AvailableOrders(MARKET.id.clone(),OrderSide::Sell)).unwrap();

        //todo: 统一数据结构
        let mut available_sell2 = available_sell.iter().map(|x|{
            BookOrder {
                id: x.id.clone(),
                account: x.account.clone(),
                side: x.side.clone(),
                price: x.price,
                amount: x.amount,
                created_at: time2unix(x.created_at.clone())
            }
         }).collect::<Vec<BookOrder>>();

        available_sell2.sort_by(|a,b|{
            a.price.partial_cmp(&b.price).unwrap()
        });

        let mut available_buy2 = available_buy.iter().map(|x|{
            BookOrder {
                id: x.id.clone(),
                account: x.account.clone(),
                side: x.side.clone(),
                price: x.price,
                amount: x.amount,
                created_at: time2unix(x.created_at.clone())
            }
        }).collect::<Vec<BookOrder>>();
        available_buy2.sort_by(|a,b|{
            a.price.partial_cmp(&b.price).unwrap().reverse()
        });

        //let available_sell = Vec::<BookOrder>::new();
        //let available_buy = Vec::<BookOrder>::new();
        EngineBook {
            buy: available_buy2,
            sell: available_sell2
        }
    });


}

#[derive(Clone, Serialize)]
pub struct AddBook {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

#[derive(Clone, Serialize, Debug)]
pub struct AddBook2 {
    pub asks: HashMap<U256, I256>,
    pub bids: HashMap<U256, I256>,
}

#[derive(RustcEncodable, Clone, Serialize)]
pub struct LastTrade {
    id: String,
    price: f64,
    amount: f64,
    taker_side: String,
    updated_at: u64,
}

#[derive(Clone, Serialize, Debug)]
pub struct LastTrade2 {
    id: String,
    price: f64,
    amount: f64,
    height: i32,
    taker_side: OrderSide,
}

//block content logs [NewOrderFilter { user: 0xfaa56b120b8de4597cf20eff21045a9883e82aad, base_token: "BTC", quote_token: "USDT", amount: 3, price: 4 }]
/**
#[derive(RustcEncodable, Clone, Serialize,Debug,Deserialize)]
struct NewOrderFilter2 {
    user: Address,
    base_token: String,
    quote_token: String,
    side: String,
    amount: u64,
    price: u64,
}
 */

abigen!(
    SimpleContract,
    //"../contract/chemix_trade_abi.json",
    "../contract/ChemixStorage.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub fn sign() -> Result<()> {
    println!("in sign");
    Ok(())
}

async fn get_balance() -> Result<()> {
    let host = "https://mainnet.infura.io/v3/8b4e814a07474456828cc110195adca2";
    let provider_http = Provider::<Http>::try_from(host).unwrap();
    let addr = "90a97d253608B2090326097a44eA289d172c30Ec".parse().unwrap();
    let union = NameOrAddress::Address(addr);
    let balance_before = provider_http.get_balance(union, None).await?;
    eprintln!("balance {}", balance_before);
    Ok(())
}

fn gen_depth_from_cancel_orders(pending_thaws: Vec<Thaws>) -> AddBook2 {
    let mut add_depth = AddBook2 {
        asks: HashMap::<U256, I256>::new(),
        bids: HashMap::<U256, I256>::new(),
    };

    let mut update_depth = |x: Thaws| {
        let amount = I256::try_from(x.amount).unwrap();
        match x.side {
            Side::Buy => {
                match add_depth.bids.get_mut(&x.price) {
                    None => {
                        add_depth.bids.insert(x.price, -amount);
                    }
                    Some(mut tmp1) => {
                        tmp1 = &mut tmp1.sub(amount);
                    }
                };
            }
            Side::Sell => {
                match add_depth.asks.get_mut(&x.price) {
                    None => {
                        add_depth.asks.insert(x.price, -amount);
                    }
                    Some(mut tmp1) => {
                        tmp1 = &mut tmp1.sub(amount);
                    }
                };
            }
        }
    };

    for pending_thaw in pending_thaws.clone() {
        update_depth(pending_thaw);
    }
    add_depth
}

fn gen_depth_from_raw(add_depth: AddBook2) -> AddBook {
    let asks2 = add_depth
        .asks
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
            let user_volume = if y < &I256::from(0u32) {
                u256_to_f64(y.abs().into_raw(), BaseTokenDecimal) * -1.0f64
            } else {
                u256_to_f64(y.abs().into_raw(), BaseTokenDecimal)
            };
            (user_price, user_volume)
        })
        .filter(|(p, v)| p != &0.0 && v != &0.0)
        .collect::<Vec<(f64, f64)>>();

    let bids2 = add_depth
        .bids
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
            let user_volume = if y < &I256::from(0u32) {
                u256_to_f64(y.abs().into_raw(), BaseTokenDecimal) * -1.0f64
            } else {
                u256_to_f64(y.abs().into_raw(), BaseTokenDecimal)
            };
            (user_price, user_volume)
        })
        .filter(|(p, v)| p != &0.0 && v != &0.0)
        .collect::<Vec<(f64, f64)>>();

    AddBook {
        asks: asks2,
        bids: bids2,
    }
}

fn gen_agg_trade_from_raw(trades: Vec<TradeInfo>) -> Vec<LastTrade2> {
    trades
        .into_iter()
        .map(|x| LastTrade2 {
            id: x.id,
            price: u256_to_f64(x.price, crate::MARKET.quote_contract_decimal),
            amount: u256_to_f64(x.amount, crate::MARKET.base_contract_decimal),
            height: -1,
            taker_side: x.taker_side,
        })
        .filter(|x| x.price != 0.0 && x.amount != 0.0)
        .collect::<Vec<LastTrade2>>()
}

async fn send_depth_message(depth: AddBook, arc_queue: Arc<RwLock<Rsmq>>) {
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

async fn send_agg_trade_message(agg_trade: Vec<LastTrade2>, arc_queue: Arc<RwLock<Rsmq>>) {
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
    let arc_queue = Arc::new(RwLock::new(queue));
    let arc_queue_cancel = arc_queue.clone();
    //不考虑安全性,随便写个私钥
    let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
    let chemix_storage_client = ChemixContractClient::<Storage>::new(pri_key);
    let chemix_storage_client = Arc::new(RwLock::new(chemix_storage_client));

    rayon::scope(|s| {
        //监听合约事件（新建订单和取消订单），将其发送到相应处理模块
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                //let watcher = gen_watcher().await;
                //let mut stream = watcher.provide.watch_blocks().await.unwrap();
                let last_order = list_orders(OrderFilter::GetLastOne).unwrap();
                let mut last_process_height = if last_order.is_empty() {
                    get_current_block().await
                } else {
                    last_order[0].block_height
                };

                loop {
                    let current_height = get_current_block().await;
                    assert!(current_height >= last_process_height);
                    if current_height - last_process_height <= CONFIRM_HEIGHT {
                        info!("current chain height {},wait for new block", current_height);
                        tokio::time::sleep(time::Duration::from_millis(1000)).await;
                    } else {
                        info!("current_book {:#?}", crate::BOOK.lock().unwrap());
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
                            //取消订单
                            let new_cancel_orders = chemix_storage_client
                                .clone()
                                .write()
                                .unwrap()
                                .filter_new_cancel_order_created_event(
                                    block_hash.clone(),
                                    crate::MARKET.base_token_address.clone(),
                                    crate::MARKET.quote_token_address.clone(),
                                )
                                .await
                                .unwrap();
                            info!("new_cancel_orders_event {:?}", new_cancel_orders);
                            let legal_orders = cancel(new_cancel_orders.clone());
                            if legal_orders.is_empty() {
                                info!(
                                    "Not found legal_cancel orders created at height {}",
                                    height
                                );
                            } else {
                                let mut pending_thaws = Vec::new();
                                for cancel_order in legal_orders {
                                    //todo: 重复list_order了
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
                                        updated_at: get_current_time(),
                                    };
                                    //todo: 批量更新
                                    update_order(&update_info);
                                    pending_thaws.push(Thaws::new(
                                        order.id.clone(),
                                        Address::from_str(order.account.as_str()).unwrap(),
                                        order.market_id.clone(),
                                        order.available_amount,
                                        order.price,
                                        order.side.clone(),
                                    ));
                                }
                                insert_thaws(pending_thaws.clone());
                                let raw_depth = gen_depth_from_cancel_orders(pending_thaws);
                                let depth = gen_depth_from_raw(raw_depth);
                                send_depth_message(depth, arc_queue_cancel.clone()).await;
                                //todo: 推送取消的深度
                            }

                            //过滤新下订单
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

                            if new_orders.is_empty() {
                                info!("Not found new order created at height {}", height);
                            } else {
                                let mut db_new_orders = Vec::new();
                                for order in new_orders {
                                    let base_decimal =
                                        crate::MARKET.base_contract_decimal as u32;
                                    let raw_amount = if base_decimal > order.num_power {
                                        order.amount
                                            * teen_power!(base_decimal - order.num_power)
                                    } else {
                                        order.amount
                                            * teen_power!(order.num_power - base_decimal)
                                    };
                                    //todo: 非法数据过滤
                                    db_new_orders.push(OrderInfo::new(
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
                                event_sender
                                    .send(db_new_orders)
                                    .expect("failed to send orders");
                            }
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
                let mut orders: Vec<OrderInfo> =
                    event_receiver.recv().expect("failed to recv book order");
                println!(
                    "[listen_blocks: receive] New order Event {:?},base token {:?}",
                    orders[0].id, orders[0].side
                );
                //TODO: matched order
                //update OrderBook
                let mut add_depth = AddBook2 {
                    asks: HashMap::<U256, I256>::new(),
                    bids: HashMap::<U256, I256>::new(),
                };

                let mut db_trades = Vec::<TradeInfo>::new();
                //market_orders的移除或者减少
                let mut db_marker_orders_reduce = HashMap::<String, U256>::new();
                for (index, db_order) in orders.iter_mut().enumerate() {
                    let _matched_amount = match_order(
                        db_order,
                        &mut db_trades,
                        &mut add_depth,
                        &mut db_marker_orders_reduce,
                    );

                    info!(
                        "index {},taker amount {},matched-amount {}",
                        index, db_order.amount, _matched_amount
                    );
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
                    //tmp code: 校验数据准确性用，后边移除
                    assert_eq!(_matched_amount, db_order.amount - db_order.available_amount);
                    db_order.matched_amount = db_order.amount - db_order.available_amount;
                    info!(
                        "finished match_order index {},and status {:?},status_str={},",
                        index,
                        db_order.status,
                        db_order.status.as_str()
                    );
                }
                error!("db_trades = {:?}", db_trades);

                //todo: 和下边的db操作的事务一致性处理
                if !db_trades.is_empty() {
                    insert_trades(&mut db_trades);
                }
                insert_order(orders.clone());
                //update marker orders
                info!("db_marker_orders_reduce {:?}", db_marker_orders_reduce);
                for orders in db_marker_orders_reduce {
                    let market_orders =
                        list_orders(OrderFilter::ById(orders.0.clone())).unwrap();
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
                        updated_at: get_current_time(),
                    };
                    //todo: 批量更新
                    update_order(&update_info);
                }

                //撮合信息推到ws模块
                let depth = gen_depth_from_raw(add_depth);
                let rt = Runtime::new().unwrap();
                let arc_queue = arc_queue.clone();
                rt.block_on(async move {
                    send_depth_message(depth, arc_queue.clone()).await;
                    let trades = gen_agg_trade_from_raw(db_trades);
                    if !trades.is_empty() {
                        info!("agg_trade {:?}", trades);
                        send_agg_trade_message(trades, arc_queue.clone()).await;
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
    let queue = Queue::regist(vec![QueueType::Depth, QueueType::Trade, QueueType::Thaws]).await;
    info!("market {}", MARKET.base_token_address);
    info!("initial book {:#?}", crate::BOOK.lock().unwrap());
    listen_blocks(queue).await;
    Ok(())
}
