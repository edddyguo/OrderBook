pub mod order;
mod trade;
mod queue;

use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;

//use ethers::providers::Ws;
use ethers_contract_abigen::parse_address;
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use rsmq_async::{Rsmq, RsmqConnection, RsmqError};
use chemix_chain::chemix::{CancelOrderState2, ChemixContractClient, SettleValues2, ThawBalances};
use chemix_chain::chemix::SettleValues;
use chemix_chain::bsc::Node;
use std::string::String;

use serde::Serialize;
use std::convert::TryFrom;
use std::env;
use std::fmt::Debug;
use std::ops::{Add, Div, Sub};
use std::str::FromStr;
use ethers::types::Address;


use crate::order::{cancel, match_order};
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time;

use chemix_utils::time as chemix_time;
use chrono::prelude::*;
use clap::{App, Arg};

use chemix_models::order::{get_order, insert_order, list_available_orders, update_order, EngineOrder, OrderInfo, Side, UpdateOrder, Status as OrderStatus, BookOrder, Status};
use chemix_models::trade::{insert_trades, TradeInfo};
use chemix_utils::algorithm::sha256;
use chemix_utils::math::{narrow, MathOperation, u256_to_f64};
use chemix_utils::time::get_current_time;
use chemix_utils::time::time2unix;
use ethers_core::abi::ethereum_types::U64;
use ethers_core::types::BlockId::Hash;
use chemix_models::api::get_markets;
use chemix_models::order::IdOrIndex::{Id, Index};
use chemix_utils::env::CONF as ENV_CONF;
use crate::queue::Queue;

use crate::Side::{Buy, Sell};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;


static BaseTokenDecimal: u32 = 18;
static QuoteTokenDecimal: u32 = 15;

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
    static ref BOOK: Mutex<EngineBook> = Mutex::new({
        let available_sell : Vec<EngineOrder> = list_available_orders("BTC-USDT",Side::Sell);
        let available_buy : Vec<EngineOrder> = list_available_orders("BTC-USDT",Side::Buy);

        let mut available_sell2 = available_sell.iter().map(|x|{
            BookOrder {
                id: x.id.clone(),
                index: U256::from(0i8),
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




/***
#[derive(Debug, PartialEq, EthEvent)]
pub struct NewOrderEvent {
    user: Address,
    baseToken: String,
    quoteToken: String,
    amount: u64,
    price: u64,
}
 */

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

/***
#[derive(RustcEncodable, Clone, Serialize)]
pub struct MarketUpdateBook {
    id: String,
    data: AddBook,
}
 */

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
    taker_side: Side,
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

async fn listen_blocks(mut queue: Queue) -> anyhow::Result<()> {
    //let host = "https://bsc-dataseed4.ninicoin.io";
    //let host = "https://data-seed-prebsc-2-s3.binance.org:8545";
    //let host = "http://58.33.12.252:8548";
    //let host = "wss://bsc-ws-node.nariox.org:443"

    let mut last_height: U64 = U64::from(200u64);
    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let (cancel_event_sender, cancel_event_receiver) = mpsc::sync_channel(0);

    let arc_queue = Arc::new(RwLock::new(queue));
    let arc_queue = arc_queue.clone();


    //set network
    //let chemix_main_addr = "0xAfC8a33002B274F43FC56D28D515406966354388";

    let chemix_storage = ENV_CONF.chemix_storage.to_owned().unwrap();

    info!("___chemix_storage={}",chemix_storage.to_str().unwrap());
    //test2
    //let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
    //test1
    let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    //local test1
    //let pri_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let mut chemix_main_client = ChemixContractClient::new(pri_key, chemix_storage.to_str().unwrap());
    let chemix_main_client_arc = Arc::new(RwLock::new(chemix_main_client));
    let chemix_main_client_receiver = chemix_main_client_arc.clone();
    let chemix_main_client_sender = chemix_main_client_arc.clone();
    let chemix_main_client_receiver2 = chemix_main_client_arc.clone();

    let ws_url = ENV_CONF.chain_ws.to_owned().unwrap();
    let rpc_url = ENV_CONF.chain_rpc.to_owned().unwrap();

    let watcher = Node::<Ws>::new(ws_url.to_str().unwrap()).await;
    let provider_http = Node::<Http>::new(rpc_url.to_str().unwrap());


    info!("__0004");
    rayon::scope(|s| {
        //send event in new block
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut stream = watcher.gen_watcher().await.unwrap();
                while let Some(block) = stream.next().await {
                    info!("block {}",block);
                    //last_height = last_height.add(1i32);
                    let current_height = provider_http.get_block(block).await.unwrap().unwrap();
                    //todo: 处理块高异常的情况,get block from http
                    //assert_eq!(last_height.add(1u64), current_height);
                    info!("new_orders_event {:?}",current_height);

                    let new_cancel_orders = chemix_main_client_sender.clone().write().unwrap().filter_new_cancel_order_created_event(current_height).await.unwrap();
                    info!("new_cancel_orders_event {:?}",new_cancel_orders);
                    if new_cancel_orders.is_empty() {
                        info!("Not found new_cancel_orders created at height {}",current_height);
                    } else {
                        let legal_orders = cancel(new_cancel_orders.clone());
                        cancel_event_sender
                            .send(legal_orders)
                            .expect("failed to send orders");
                    }

                    let new_orders = chemix_main_client_sender.clone().write().unwrap().filter_new_order_event(current_height).await.unwrap();
                    info!("new_orders_event {:?}",new_orders);


                    if new_orders.is_empty() {
                        info!("Not found new order created at height {}",current_height);
                    } else {
                        event_sender
                            .send(new_orders)
                            .expect("failed to send orders");
                    }
                    last_height = current_height;
                }
            });
        });
        //execute thawbalance
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let cancel_orders: Vec<CancelOrderState2> = cancel_event_receiver.recv().expect("failed to send orders");
                //todo: 1、计算amount，2、上链，3、更新db
                let mut thaw_infos = Vec::new();
                for cancel_order in cancel_orders.clone() {
                    let order = get_order(Index(cancel_order.order_index.as_u32())).unwrap();
                    let market = get_markets(order.market_id.as_str());
                    let cancel_amount = order.available_amount;
                    let token_base_decimal = U256::from(10u128).pow(U256::from(18u32));
                    let (token_address,amount) = match order.side.as_str() {
                        "sell" => {
                            (market.base_token_address,order.available_amount)
                        },
                        "buy" => {
                            (market.quote_token_address,order.available_amount * order.price / token_base_decimal)
                        }
                        _ => {
                            unreachable!()
                        }
                    };
                    thaw_infos.push(ThawBalances {
                        token: Address::from_str(token_address.as_str()).unwrap(),
                        from: cancel_order.order_user,
                        amount
                    });
                }
                info!("all thaw info {:?}",thaw_infos);
                info!("start thaw balance");
                let chemix_main_client2 = chemix_main_client_receiver2.clone();

                let thaw_res = chemix_main_client2.read().unwrap().thaw_balances(thaw_infos).await.unwrap().unwrap();
                info!("finish thaw balance res:{:?}",thaw_res);


                for cancel_order in cancel_orders {
                    let order = get_order(Index(cancel_order.order_index.as_u32())).unwrap();
                    let new_status = "canceled";

                    let update_info = UpdateOrder {
                        id: order.id,
                        status: new_status.to_string(),
                        available_amount: U256::from(0i32),
                        matched_amount: order.matched_amount,
                        canceled_amount: order.available_amount,
                        updated_at: get_current_time(),
                    };
                    //todo: 批量更新
                    update_order(&update_info);
                }
            });
        });

        s.spawn(move |_| {
            loop {
                let orders: Vec<BookOrder> =
                    event_receiver.recv().expect("failed to recv columns");
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
                    let mut db_order = OrderInfo::new(order.id.clone(), order.index.clone(), "BTC-USDT".to_string(), order.account.clone(), order.side.clone(), order.price.clone(), order.amount.clone());
                    let matched_amount = match_order(order, &mut db_trades, &mut add_depth, &mut db_marker_orders_reduce);

                    error!("index={},taker_amount={},matched_amount={}",index,db_order.amount,matched_amount);
                    db_order.status = if matched_amount == db_order.amount {
                        info!("0001");
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
                    info!("finished match_order index {},and status {:?},status_str={},",index,db_order.status,db_order.status.as_str());
                    db_orders.push(db_order);
                }
                error!("db_trades = {:?}",db_trades);

                error!("gen add depth = {:?}",add_depth);

                //todo: settle traders
                let mut settle_values: HashMap<String, EnigneSettleValues> = HashMap::new();
                let token_base_decimal = U256::from(10u128).pow(U256::from(18u32));
                for trader in db_trades.clone() {
                    match trader.taker_side {
                        Buy => {
                            let taker_base_amount = I256::from_raw(trader.amount);
                            match settle_values.get_mut(&trader.taker) {
                                None => {
                                    settle_values.insert(trader.taker, EnigneSettleValues {
                                        incomeQuoteToken: I256::from(0u32),
                                        incomeBaseToken: taker_base_amount,
                                    });
                                }
                                Some(tmp1) => {
                                    tmp1.incomeBaseToken += taker_base_amount;
                                }
                            }

                            let maker_quote_amount = I256::from_raw(trader.amount * trader.price / token_base_decimal) * I256::from(-1i32);
                            match settle_values.get_mut(&trader.maker) {
                                None => {
                                    settle_values.insert(trader.maker, EnigneSettleValues {
                                        incomeQuoteToken: maker_quote_amount,
                                        incomeBaseToken: I256::from(0i32),
                                    });
                                }
                                Some(tmp1) => {
                                    tmp1.incomeQuoteToken += maker_quote_amount
                                }
                            }
                        }
                        Sell => {
                            let taker_base_amount = I256::from_raw(trader.amount) * I256::from(-1i32);
                            match settle_values.get_mut(&trader.taker) {
                                None => {
                                    settle_values.insert(trader.taker, EnigneSettleValues {
                                        incomeQuoteToken: I256::from(0u32),
                                        incomeBaseToken: taker_base_amount,
                                    });
                                }
                                Some(tmp1) => {
                                    tmp1.incomeBaseToken += taker_base_amount;
                                }
                            }

                            let maker_quote_amount = I256::from_raw(trader.amount * trader.price / token_base_decimal);
                            match settle_values.get_mut(&trader.maker) {
                                None => {
                                    settle_values.insert(trader.maker, EnigneSettleValues {
                                        incomeQuoteToken: maker_quote_amount,
                                        incomeBaseToken: I256::from(0i32),
                                    });
                                }
                                Some(tmp1) => {
                                    tmp1.incomeQuoteToken += maker_quote_amount
                                }
                            }
                        }
                    }
                }


                info!("_pre_settlement result {:#?}",settle_values);
                let settle_trades = settle_values.iter().map(|(address, settle_info)| {
                    SettleValues2 {
                        user: Address::from_str(address).unwrap(),
                        positiveOrNegative2: settle_info.incomeBaseToken.is_positive(),
                        incomeBaseToken: settle_info.incomeBaseToken.abs().into_raw(),
                        positiveOrNegative1: settle_info.incomeQuoteToken.is_positive(),
                        incomeQuoteToken: settle_info.incomeQuoteToken.abs().into_raw(),
                    }
                }).collect::<Vec<SettleValues2>>();

                info!("settle_trades {:?} ",settle_trades);


                //fixme:有revert
                let rt = Runtime::new().unwrap();
                let chemix_main_client2 = chemix_main_client_receiver.clone();
                let settlement_res = rt.block_on(async {
                    let mut receipt = Default::default();
                    loop {
                        match chemix_main_client2.read().unwrap().settlement_trades(settle_trades.clone()).await {
                            Ok(data) => {
                                receipt = data.unwrap();
                                break;
                            }
                            Err(error) => {
                                if error.to_string().contains("underpriced") {
                                    warn!("gas too low and try again");
                                    tokio::time::sleep(time::Duration::from_millis(5000)).await;
                                } else {
                                    //tmp code
                                    error!("{}",error);
                                    unreachable!()
                                }
                            }
                        }
                    }
                    receipt
                });

                //todo: 没有区块的情况？
                let height = settlement_res.block_number.unwrap().to_string().parse::<u32>().unwrap();
                //------------------
                //todo: marker orders的状态也要更新掉
                //todo: 异步落表
                //todo： 等待清算
                insert_order(db_orders);
                //update marker orders
                let u256_zero = U256::from(0i32);
                for orders in db_marker_orders_reduce {
                    let marker_order_ori = get_order(Id(orders.0.clone())).unwrap();

                    let new_matched_amount = marker_order_ori.matched_amount + orders.1;
                    let new_available_amount = marker_order_ori.available_amount - orders.1;

                    let new_status = if new_available_amount == u256_zero {
                        "full_filled".to_string()
                    } else {
                        "partial_filled".to_string()
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
                insert_trades(&mut db_trades);
                //----------------------

                let agg_trades = db_trades.iter().map(|x| {
                    let user_price = u256_to_f64(x.price, QuoteTokenDecimal);
                    let user_amount = u256_to_f64(x.amount, BaseTokenDecimal);
                    LastTrade2 {
                        price: user_price,
                        amount: user_amount,
                        height,
                        taker_side: x.taker_side.clone(),
                    }
                }
                ).filter(|x| {
                    x.price != 0.0 && x.amount != 0.0
                }).collect::<Vec<LastTrade2>>();

                info!("finished compute  agg_trades {:?},add_depth {:?}",agg_trades,add_depth);

                let asks2 = add_depth.asks.iter().map(|(x, y)| {
                    let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
                    // let user_volume = u256_to_f64(y.to_owned(), BaseTokenDecimal);
                    info!("__test_decimal_0001_{}_{}_{}",y,y.into_raw(),y.abs().into_raw());
                    let user_volume = if y < &I256::from(0u32) {
                        u256_to_f64(y.abs().into_raw(), BaseTokenDecimal) * -1.0f64
                    } else {
                        u256_to_f64(y.abs().into_raw(), BaseTokenDecimal)
                    };

                    (user_price, user_volume)
                }).filter(|(p, v)| {
                    p != &0.0 && v != &0.0
                }).collect::<Vec<(f64, f64)>>();

                let bids2 = add_depth.bids.iter().map(|(x, y)| {
                    info!("__test_decimal_0002_{}_{}_{}",y,y.into_raw(),y.abs().into_raw());
                    let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
                    let user_volume = if y < &I256::from(0u32) {
                        u256_to_f64(y.abs().into_raw(), BaseTokenDecimal) * -1.0f64
                    } else {
                        u256_to_f64(y.abs().into_raw(), BaseTokenDecimal)
                    };
                    (user_price, user_volume)
                }).filter(|(p, v)| {
                    p != &0.0 && v != &0.0
                }).collect::<Vec<(f64, f64)>>();

                let book2 = AddBook {
                    asks: asks2,
                    bids: bids2,
                };

                //let channel_update_book = channel_update_book.clone();
                //let channel_new_trade = channel_new_trade.clone();
                let arc_queue = arc_queue.clone();
                let update_book_queue = arc_queue.read().unwrap().UpdateBook.clone();
                let new_trade_queue = arc_queue.read().unwrap().NewTrade.clone();

                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let json_str = serde_json::to_string(&book2).unwrap();
                    arc_queue.write().unwrap().client
                        .send_message(update_book_queue.as_str(), json_str, None)
                        .await
                        .expect("failed to send message");

                    if !agg_trades.is_empty() {
                        let json_str = serde_json::to_string(&agg_trades).unwrap();
                        arc_queue.write().unwrap().client
                            .send_message(new_trade_queue.as_str(), json_str, None)
                            .await
                            .expect("failed to send message");
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
    let queue = Queue::new().await;
    /***
    let matches = App::new("Hellman")
        .version("1.0")
        .about("Does awesome things")
        .arg(Arg::new("pem_path")
            .about("Sets the pem file to use")
            .required(true)
            .index(1));

     */

    info!("initial book {:#?}", crate::BOOK.lock().unwrap());
    listen_blocks(queue).await;
    Ok(())
}
