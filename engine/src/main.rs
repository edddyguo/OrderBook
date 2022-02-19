mod order;
mod trade;

use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;

//use ethers::providers::Ws;
use ethers_contract_abigen::parse_address;
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use rsmq_async::{Rsmq, RsmqConnection, RsmqError};

use serde::Serialize;
use std::convert::TryFrom;
use std::env;
use std::fmt::Debug;
use std::ops::{Add, Div, Sub};

use crate::order::{match_order, BookOrder};
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::time;

use chemix_utils::time as chemix_time;
use chrono::prelude::*;

use chemix_models::order::{get_order, insert_order, list_available_orders, update_order, EngineOrder, OrderInfo, Side, UpdateOrder, Status as OrderStatus};
use chemix_models::trade::{insert_trades, TradeInfo};
use chemix_utils::algorithm::sha256;
use chemix_utils::math::{narrow, MathOperation};
use chemix_utils::time::get_current_time;
use chemix_utils::time::time2unix;
use ethers_core::abi::ethereum_types::U64;

use crate::Side::{Buy, Sell};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;



static QuoteTokenDecimal : u8 = 11; //11 - 8
static BaseTokenDecimal : u8 = 22; //22 - 8

#[derive(Clone, Serialize, Debug)]
struct EngineBook {
    pub buy: Vec<BookOrder>,
    pub sell: Vec<BookOrder>,
}

lazy_static! {
    static ref BOOK: Mutex<EngineBook> = Mutex::new({
        let available_sell : Vec<EngineOrder> = list_available_orders("BTC-USDT",Side::Sell);
        let available_buy : Vec<EngineOrder> = list_available_orders("BTC-USDT",Side::Buy);

        let available_sell2 = available_sell.iter().map(|x|{
            BookOrder {
                id: x.id.clone(),
                account: x.account.clone(),
                side: x.side.clone(),
                price: x.price,
                amount: x.amount,
                created_at: time2unix(x.created_at.clone())
            }
         }).collect::<Vec<BookOrder>>();

        let available_buy2 = available_buy.iter().map(|x|{
            BookOrder {
                id: x.id.clone(),
                account: x.account.clone(),
                side: x.side.clone(),
                price: x.price,
                amount: x.amount,
                created_at: time2unix(x.created_at.clone())
            }
        }).collect::<Vec<BookOrder>>();

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
    pub asks: Vec<(U256,U256)>,
    pub bids: Vec<(U256,U256)>,
}

#[derive(Clone, Serialize, Debug)]
pub struct AddBook2 {
    pub asks: HashMap<U256, U256>,
    pub bids: HashMap<U256, U256>,
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
    price: U256,
    amount: U256,
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

async fn check_queue(name: &str) {
    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");
    let attributes = rsmq.get_queue_attributes(name).await;
    match attributes {
        Ok(_) => {
            println!("queue already exist");
        }
        Err(RsmqError::QueueNotFound) => {
            println!("test2 not found");
            rsmq.create_queue(name, None, None, None)
                .await
                .expect("failed to create queue");
        }
        _ => {
            unreachable!()
        }
    }
}

async fn listen_blocks() -> anyhow::Result<()> {
    //let host = "https://bsc-dataseed4.ninicoin.io";
    //let host = "https://data-seed-prebsc-2-s3.binance.org:8545";
    let host = "http://58.33.12.252:8548";


    let provider_http = Provider::<Http>::try_from(host).unwrap();

    let rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");

    let channel_update_book = match env::var_os("CHEMIX_MODE") {
        None => "update_book_local".to_string(),
        Some(mist_mode) => {
            format!("update_book_{}", mist_mode.into_string().unwrap())
        }
    };

    let channel_new_trade = match env::var_os("CHEMIX_MODE") {
        None => "new_trade_local".to_string(),
        Some(mist_mode) => {
            format!("new_trade_{}", mist_mode.into_string().unwrap())
        }
    };

    info!("__0007");
    check_queue(channel_update_book.as_str()).await;
    check_queue(channel_new_trade.as_str()).await;
    info!("__0008");
    //todo: wss://bsc-ws-node.nariox.org:443
    /***
    //let ws = Ws::connect("wss://bsc-ws-node.nariox.org:443/").await.unwrap();
    let ws = Ws::connect("ws://192.168.1.158:7548/").await.unwrap();
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?;
    while let Some(block) = stream.next().await {
        let block_content = provider_http.get_block(block).await.unwrap();
        println!("block content {:?}",block_content);
    }
     */
    //test2
    let wallet = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43"
        .parse::<LocalWallet>()
        .unwrap();
    //private network start
    let mut height: U64 = U64::from(33992u64);
    let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
    let client = Arc::new(client);

    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let arc_rsmq = Arc::new(RwLock::new(rsmq));
    let arc_rsmq2 = arc_rsmq.clone();
    info!("__0004");
    rayon::scope(|s| {
        //send event in new block
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            info!("__0005");
            rt.block_on(async move {
                loop {
                    let block_content = provider_http.get_block(height).await.unwrap();
                    if block_content.is_none() {
                        tokio::time::sleep(time::Duration::from_secs(2)).await;
                        info!("block not found at heigh {},and wait a moment",height);
                    } else {
                        info!("find a new order at height{}",height);
                        height = height.add(1u64);
                        let addr = parse_address("0xFFc6817E1c8960b278CCb5e47c2e6D3ae9Fed620")
                            .unwrap();
                        let contract = SimpleContract::new(addr, client.clone());
                        info!("test_filter_time_0006");
                        let new_orders: Vec<NewOrderCreatedFilter> = contract
                            .new_order_created_filter()
                            .from_block(height.as_u64())
                            .query()
                            .await
                            .unwrap();
                        info!("test_filter_time_0007");

                        let new_orders2 = new_orders
                            .iter()
                            .map(|x| {
                                let now = Local::now().timestamp_millis() as u64;
                                let order_json = format!(
                                    "{}{}",
                                    serde_json::to_string(&x).unwrap(),
                                    now
                                );
                                let order_id = sha256(order_json);
                                let side = match x.order_type {
                                    true => Buy,
                                    false => Sell,
                                };
                                info!("__0001_{:#?}",x);
                                //let price =  x.limit_price.div(U256::from(10i32).pow(U256::from(BaseTokenDecimal - QuoteTokenDecimal))).as_u64();
                                //let amount = x.order_amount.div(U256::from(10i32).pow(U256::from(QuoteTokenDecimal))).as_u64();
                                BookOrder {
                                    id: order_id,
                                    account: x.order_user.to_string(),
                                    side,
                                    price: x.limit_price,
                                    amount: x.order_amount,
                                    created_at: now,
                                }
                            })
                            .collect::<Vec<BookOrder>>();
                        if new_orders2.is_empty() {
                            info!("Not found new order created at height {}",height);
                        }else {
                            event_sender
                                .send(new_orders2)
                                .expect("failed to send orders");
                        }
                    }
                }
            });
        });
        s.spawn(move |_| {
            loop {
                let arc_rsmq = arc_rsmq.clone();
                let orders: Vec<BookOrder> =
                    event_receiver.recv().expect("failed to recv columns");
                println!(
                    "[listen_blocks: receive] New order Event {:?},base token {:?}",
                    orders[0].id, orders[0].side
                );
                //TODO: matched order
                //update OrderBook
                let mut add_depth = AddBook2 {
                    asks: HashMap::<U256,U256>::new(),
                    bids: HashMap::<U256,U256>::new(),
                };

                let mut db_trades = Vec::<TradeInfo>::new();
                let mut db_orders = Vec::<OrderInfo>::new();
                //market_orders的移除或者减少
                let mut db_marker_orders_reduce = HashMap::<String,U256>::new();
                let u256_zero = U256::from(0i32);

                for  (index,order) in orders.into_iter().enumerate() {
                    let mut db_order = OrderInfo::new(order.id.clone(),"BTC-USDT".to_string(),order.account.clone(),order.side.clone(),order.price.clone(),order.amount.clone());
                    let matched_amount = match_order(order, &mut db_trades, &mut add_depth,&mut db_marker_orders_reduce);

                    error!("index={},taker_amount={},matched_amount={}",index,db_order.amount,matched_amount);
                    db_order.status = if matched_amount == db_order.amount {
                        info!("0001");
                        OrderStatus::FullFilled
                    }else if  matched_amount != u256_zero && matched_amount < db_order.amount{
                        OrderStatus::PartialFilled
                    }else if matched_amount == u256_zero{
                        OrderStatus::Pending
                    }else {
                        unreachable!()
                    };
                    db_order.matched_amount = matched_amount;
                    db_order.available_amount = db_order.amount.sub(matched_amount);
                    info!("finished match_order index {},and status {:?},status_str={},",index,db_order.status,db_order.status.as_str());
                    db_orders.push(db_order);
                }
                error!("db_trades = {:?}",db_trades);


                let agg_trades = db_trades.iter().map(|x|
                    LastTrade2 {
                        price: x.price,
                        amount: x.amount,
                        taker_side: x.taker_side.clone(),
                    }
                ).collect::<Vec<LastTrade2>>();


                //------------------
                //todo: marker orders的状态也要更新掉
                //todo: 异步落表
                //todo： 等待清算
                insert_order(db_orders);
                //update marker orders
                let u256_zero = U256::from(0i32);
                for orders in db_marker_orders_reduce {
                    let marker_order_ori = get_order(orders.0.as_str()).unwrap();

                    let new_matched_amount = marker_order_ori.matched_amount + orders.1;
                    let new_available_amount = marker_order_ori.available_amount - orders.1;

                    let new_status = if new_available_amount == u256_zero {
                        "full_filled".to_string()
                    }else{
                        "partial_filled".to_string()
                    };

                    let update_info = UpdateOrder{
                        id: orders.0,
                        status: new_status,
                        available_amount: new_available_amount,
                        canceled_amount: marker_order_ori.canceled_amount,
                        matched_amount: new_matched_amount,
                        updated_at: get_current_time()
                    };
                    update_order(&update_info);

                }
                insert_trades(&mut db_trades);
                //----------------------

                info!("finished compute  agg_trades {:?},add_depth {:?}",agg_trades,add_depth);

                let asks2 = add_depth.asks.iter().map(|(x,y)| {
                    (x.to_owned(),y.to_owned())
                }).collect::<Vec<(U256,U256)>>();

                let bids2 = add_depth.bids.iter().map(|(x,y)| {
                        (x.to_owned(),y.to_owned())
                }).collect::<Vec<(U256,U256)>>();

                let book2 = AddBook {
                    asks:asks2,
                    bids:bids2,
                };

                let channel_update_book = channel_update_book.clone();
                let channel_new_trade = channel_new_trade.clone();
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let json_str = serde_json::to_string(&book2).unwrap();
                    //let json_str = serde_json::to_string(&add_depth).unwrap();
                    arc_rsmq
                        .write()
                        .unwrap()
                        .send_message(channel_update_book.as_str(), json_str, None)
                        .await
                        .expect("failed to send message");

                    if !agg_trades.is_empty() {
                        let json_str = serde_json::to_string(&agg_trades).unwrap();
                        arc_rsmq
                            .write()
                            .unwrap()
                            .send_message(channel_new_trade.as_str(), json_str, None)
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

    info!("initial book {:#?}", crate::BOOK.lock().unwrap());
    listen_blocks().await;
    Ok(())
}
