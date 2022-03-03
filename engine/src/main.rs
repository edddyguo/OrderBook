pub mod order;
mod queue;
mod trade;

use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;

//use ethers::providers::Ws;

use chemix_chain::chemix::ChemixContractClient;
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use rsmq_async::RsmqConnection;

use chemix_chain::bsc::Node;
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

use tokio::runtime::Runtime;

use clap::{App, Arg};

use chemix_models::order::{
    get_order, insert_order, list_available_orders, update_order, BookOrder, EngineOrder,
    OrderInfo, UpdateOrder,
};
use chemix_models::trade::{insert_trades, TradeInfo};

use common::utils::math::u256_to_f64;
use common::utils::time::get_current_time;
use common::utils::time::time2unix;
use ethers_core::abi::ethereum_types::U64;

use chemix_models::api::get_markets;
use chemix_models::order::IdOrIndex::{Id, Index};
//use common::env::CONF as ENV_CONF;
use crate::queue::Queue;
use common::env::CONF as ENV_CONF;

use common::types::order::Status as OrderStatus;

use common::types::order::Side as OrderSide;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

static BaseTokenDecimal: u32 = 18;
static QuoteTokenDecimal: u32 = 15;
use chemix_models::api::MarketInfo;
use chemix_models::thaws::{insert_thaws, Thaws};

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
    let market = get_markets(market_id);
        market
    };

    static ref BOOK: Mutex<EngineBook> = Mutex::new({
        let available_sell : Vec<EngineOrder> = list_available_orders(MARKET.id.as_str(),OrderSide::Sell);
        let available_buy : Vec<EngineOrder> = list_available_orders(MARKET.id.as_str(),OrderSide::Buy);

        //todo: 统一数据结构
        let mut available_sell2 = available_sell.iter().map(|x|{
            BookOrder {
                id: x.id.clone(),
                index: U256::from(0i8),
                hash_data: "".to_string(),
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
                index: U256::from(0i8), //todo
                hash_data: "".to_string(),
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
    price: f64,
    amount: f64,
    height: u32,
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

async fn listen_blocks(queue: Queue) -> anyhow::Result<()> {
    //let host = "https://bsc-dataseed4.ninicoin.io";
    //let host = "https://data-seed-prebsc-2-s3.binance.org:8545";
    //let host = "http://58.33.12.252:8548";
    //let host = "wss://bsc-ws-node.nariox.org:443"
    //todo: 重启从上次结束的块开始扫
    let mut last_height: U64 = U64::from(200u64);
    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let arc_queue = Arc::new(RwLock::new(queue));
    let arc_queue = arc_queue.clone();

    let chemix_storage = ENV_CONF.chemix_storage.to_owned().unwrap();
    //test1
    //let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";

    let chemix_main_client =
        ChemixContractClient::new(pri_key, chemix_storage.to_str().unwrap());
    let chemix_main_client_arc = Arc::new(RwLock::new(chemix_main_client));
    let _chemix_main_client_receiver = chemix_main_client_arc.clone();
    let chemix_main_client_sender = chemix_main_client_arc.clone();
    let _chemix_main_client_receiver2 = chemix_main_client_arc.clone();
    let ws_url = ENV_CONF.chain_ws.to_owned().unwrap();
    let rpc_url = ENV_CONF.chain_rpc.to_owned().unwrap();

    let watcher = Node::<Ws>::new(ws_url.to_str().unwrap()).await;
    let provider_http = Node::<Http>::new(rpc_url.to_str().unwrap());

    rayon::scope(|s| {
        //监听合约事件（新建订单和取消订单），将其发送到相应处理模块
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut stream = watcher.gen_watcher().await.unwrap();
                while let Some(block) = stream.next().await {
                    info!("block {}", block);
                    info!("current_book {:#?}", crate::BOOK.lock().unwrap());
                    //last_height = last_height.add(1i32);
                    let current_height = provider_http.get_block(block).await.unwrap().unwrap();
                    //todo: 处理块高异常的情况,get block from http
                    //assert_eq!(last_height.add(1u64), current_height);
                    info!("new_orders_event {:?}", current_height);

                    let new_cancel_orders = chemix_main_client_sender
                        .clone()
                        .write()
                        .unwrap()
                        .filter_new_cancel_order_created_event(current_height,crate::MARKET.base_token_address.clone(),crate::MARKET.quote_token_address.clone())
                        .await
                        .unwrap();
                    info!("new_cancel_orders_event {:?}", new_cancel_orders);
                    let legal_orders = cancel(new_cancel_orders.clone());
                    if legal_orders.is_empty() {
                        info!(
                            "Not found legal_cancel orders created at height {}",
                            current_height
                        );
                    } else {
                        let mut pending_thaws = Vec::new();
                        for cancel_order in legal_orders {
                            let order =
                                get_order(Index(cancel_order.order_index.as_u32())).unwrap();
                            let update_info = UpdateOrder {
                                id: order.id.clone(),
                                status: OrderStatus::Canceled,
                                available_amount: U256::from(0i32),
                                matched_amount: order.matched_amount,
                                canceled_amount: order.available_amount,
                                updated_at: get_current_time(),
                            };
                            //todo: 批量更新
                            update_order(&update_info);
                            pending_thaws.push(Thaws::new(
                                order.id.clone(),
                                Address::from_str(order.account.as_str()).unwrap(),
                                order.market_id,
                                order.available_amount,
                                order.price,
                                order.side.clone(),
                            ));
                        }
                        insert_thaws(pending_thaws);
                    }

                    let new_orders = chemix_main_client_sender
                        .clone()
                        .write()
                        .unwrap()
                        .filter_new_order_event(current_height,crate::MARKET.base_token_address.clone(),crate::MARKET.quote_token_address.clone())
                        .await
                        .unwrap();
                    info!("new_orders_event {:?}", new_orders);

                    if new_orders.is_empty() {
                        info!("Not found new order created at height {}", current_height);
                    } else {
                        event_sender
                            .send(new_orders)
                            .expect("failed to send orders");
                    }
                    last_height = current_height;
                }
            });
        });

        s.spawn(move |_| {
            loop {
                let orders: Vec<BookOrder> =
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
                let mut db_orders = Vec::<OrderInfo>::new();
                //market_orders的移除或者减少
                let mut db_marker_orders_reduce = HashMap::<String, U256>::new();
                let u256_zero = U256::from(0i32);

                for (index, order) in orders.into_iter().enumerate() {
                    let mut db_order = OrderInfo::new(
                        order.id.clone(),
                        order.index.clone(),
                        order.hash_data.clone(),
                        MARKET.id.clone(),
                        order.account.clone(),
                        order.side.clone(),
                        order.price.clone(),
                        order.amount.clone(),
                    );
                    let matched_amount = match_order(
                        order,
                        &mut db_trades,
                        &mut add_depth,
                        &mut db_marker_orders_reduce,
                    );

                    error!(
                        "index {},taker amount {},matched-amount {}",
                        index, db_order.amount, matched_amount
                    );
                    db_order.status = if matched_amount == db_order.amount {
                        OrderStatus::FullFilled
                    } else if matched_amount != u256_zero && matched_amount < db_order.amount {
                        OrderStatus::PartialFilled
                    } else if matched_amount == u256_zero {
                        OrderStatus::Pending
                    } else {
                        unreachable!()
                    };
                    db_order.matched_amount = matched_amount;
                    db_order.available_amount = db_order.amount.sub(matched_amount);
                    info!(
                        "finished match_order index {},and status {:?},status_str={},",
                        index,
                        db_order.status,
                        db_order.status.as_str()
                    );
                    db_orders.push(db_order);
                }
                error!("db_trades = {:?}", db_trades);

                //todo: 和下边的db操作的事务一致性处理
                if !db_trades.is_empty() {
                    insert_trades(&mut db_trades);
                }
                insert_order(db_orders);
                //update marker orders
                let u256_zero = U256::from(0i32);
                for orders in db_marker_orders_reduce {
                    let marker_order_ori = get_order(Id(orders.0.clone())).unwrap();

                    let new_matched_amount = marker_order_ori.matched_amount + orders.1;
                    let new_available_amount = marker_order_ori.available_amount - orders.1;

                    let new_status = if new_available_amount == u256_zero {
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

                info!("finished compute  ,add_depth {:?}", add_depth);
                let asks2 = add_depth
                    .asks
                    .iter()
                    .map(|(x, y)| {
                        let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
                        // let user_volume = u256_to_f64(y.to_owned(), BaseTokenDecimal);
                        info!(
                            "__test_decimal_0001_{}_{}_{}",
                            y,
                            y.into_raw(),
                            y.abs().into_raw()
                        );
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
                        info!(
                            "__test_decimal_0002_{}_{}_{}",
                            y,
                            y.into_raw(),
                            y.abs().into_raw()
                        );
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

                let book2 = AddBook {
                    asks: asks2,
                    bids: bids2,
                };

                //todo: 放在luanch模块在交易确认后推送？
                if db_trades.is_empty() {
                    let mut market_add_depth = HashMap::new();
                    market_add_depth.insert(MARKET.id.clone(),book2);
                    let arc_queue = arc_queue.clone();
                    let update_book_queue = arc_queue.read().unwrap().UpdateBook.clone();
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        let json_str = serde_json::to_string(&market_add_depth).unwrap();
                        arc_queue
                            .write()
                            .unwrap()
                            .client
                            .send_message(update_book_queue.as_str(), json_str, None)
                            .await
                            .expect("failed to send message");
                    });
                }
            }
        });
    });

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let queue = Queue::new().await;
    info!("market {}", MARKET.base_token_address);
    info!("initial book {:#?}", crate::BOOK.lock().unwrap());
    listen_blocks(queue).await;
    Ok(())
}
