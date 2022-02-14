mod order;
mod trade;

use std::collections::HashMap;
use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_contract_abigen::{parse_address, Address};
use ethers_providers::{Http, Middleware, Provider, StreamExt, Ws};
use rsmq_async::{Rsmq, RsmqConnection, RsmqError, RsmqQueueAttributes};
use rustc_serialize::json;
use serde::Serialize;
use std::convert::TryFrom;
use std::env;
use std::fmt::Debug;
use std::ops::Add;
use std::str::FromStr;
use std::sync::{mpsc, Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::time;
use std::sync::Mutex;
use crate::order::{BookOrder, EventOrder, match_order, Side};

use chrono::offset::LocalResult;
use chrono::prelude::*;
use utils::{time as chemix_time,algorithm};
use ethers::{prelude::*};
use utils::math::{MathOperation, narrow};
use ethers_core::abi::ethereum_types::{U256, U64};
use chemix_models::engine::{insert_order, OrderInfo};
use utils::algorithm::sha256;
use crate::order::Status::{FullFilled, PartialFilled};
use crate::Side::{Buy, Sell};


#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;



#[derive(Clone, Serialize,Debug)]
struct EngineBook {
    pub buy: Vec<BookOrder>,
    pub sell: Vec<BookOrder>,
}

lazy_static! {
    static ref BOOK: Mutex<EngineBook> = Mutex::new({
        info!("lazy_static--postgres");
        //let available_sell_orders = postgresql::list_available_orders("sell", market);
        //let available_buy_orders = postgresql::list_available_orders("buy", market);
        let available_sell = Vec::<BookOrder>::new();
        let available_buy = Vec::<BookOrder>::new();
        EngineBook {
            buy: available_buy,
            sell: available_sell
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

#[derive(RustcEncodable, Clone, Serialize)]
pub struct AddBook {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

#[derive(RustcEncodable, Clone, Serialize,Debug)]
pub struct AddBook2 {
    pub asks: HashMap<u64,u64>,
    pub bids: HashMap<u64,u64>,
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

#[derive(RustcEncodable, Clone, Serialize,Debug)]
pub struct LastTrade2 {
    price: u64,
    amount: u64,
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
    "../contract/chemix_trade_abi.json",
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
    //testnet
    let host = "https://data-seed-prebsc-2-s3.binance.org:8545";

    let provider_http = Provider::<Http>::try_from(host).unwrap();

    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");

    let channel_update_book = match env::var_os("CHEMIX_MODE") {
        None => {
            "update_book_local".to_string()
        }
        Some(mist_mode) => {
            format!("update_book_{}",mist_mode.into_string().unwrap())
        }
    };

    let channel_new_trade = match env::var_os("CHEMIX_MODE") {
        None => {
            "new_trade_local".to_string()
        }
        Some(mist_mode) => {
            format!("new_trade_{}",mist_mode.into_string().unwrap())
        }
    };

    check_queue(channel_update_book.as_str()).await;
    check_queue(channel_new_trade.as_str()).await;


    //todo: wss://bsc-ws-node.nariox.org:443
    /***
    let ws = Ws::connect("wss://bsc-ws-node.nariox.org:443/").await.unwrap();
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?;
    while let Some(block) = stream.next().await {
        let block_content = provider_http.get_block(block).await.unwrap();
        println!("block content {:?}",block_content);
    }
     */
    let wallet = "1b03a06c4a89d570a8f1d39e9ff0be8891f7657898675f11585aa7ec94fe2d12"
        .parse::<LocalWallet>()
        .unwrap();
    let address = wallet.address();
    println!("wallet address {:?}", address);
    //let mut height = provider_http.get_block_number().await.unwrap();
    //166475590u64
    //16477780u64
    let mut height: U64 = U64::from(16647865u64);
    //let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
    //let client = Arc::new(client);

    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let mut arc_rsmq = Arc::new(RwLock::new(rsmq));
    let arc_rsmq2 = arc_rsmq.clone();
    rayon::scope(|s| {
        //send event in new block
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    //let block_content = provider_http.get_block(height).await.unwrap();
                    //if block_content.is_none() {
                    if false {
                        tokio::time::sleep(time::Duration::from_secs(2)).await;
                        println!("block not found,and wait a moment");
                    } else {
                        let addr = parse_address("0xE41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C")
                            .unwrap();
                        //let contract = SimpleContract::new(addr, client.clone());
                        /**
                        let logs: Vec<NewOrderFilter> = contract
                            .new_order_filter()
                            .from_block(height.as_u64())
                            .query()
                            .await
                            .unwrap();
                        event_sender.send(logs).expect("failed to send orders");
                         */
                        //tmp code, 压力测试也可以在这里,链上tps受限
                        let channel_bot = match env::var_os("CHEMIX_MODE") {
                            None => {
                                "bot_local".to_string()
                            }
                            Some(mist_mode) => {
                                format!("bot_{}",mist_mode.into_string().unwrap())
                            }
                        };
                        check_queue(channel_bot.as_str()).await;
                        let rsmq = arc_rsmq2.clone();
                        'listen_new_order : loop{
                            let message = rsmq.write().unwrap()
                                .receive_message::<String>(channel_bot.as_str(), None)
                                .await
                                .expect("cannot receive message");
                            if let Some(message) = message {
                                println!("receive new message {:?}", message.message);
                                let new_orders: Vec<NewOrderFilter> = serde_json::from_str(&message.message).unwrap();
                                println!("receive new order {:?} at {}", new_orders,chemix_time::get_current_time());
                                //    event NewOrder(address user, string baseToken, string quoteToken ,string side, uint amount, uint price);
                                let new_orders = new_orders.iter().map(|x | {
                                    let now = Local::now().timestamp_millis() as u64;
                                    let order_json = format!("{}{}",serde_json::to_string(&x).unwrap(),now);
                                    let order_id = sha256(order_json);
                                    let side = match x.side.as_str() {
                                        "sell" => Sell,
                                        "buy" => Buy,
                                        _ => unreachable!()
                                    };
                                    BookOrder {
                                        id: order_id,
                                        account: x.user.to_string(),
                                        side,
                                        price: x.price.as_u64(),
                                        amount: x.amount.as_u64(),
                                        created_at: now,
                                    }
                                },).collect::<Vec<BookOrder>>();
                                event_sender.send(new_orders).expect("failed to send orders");
                                rsmq.write().unwrap().delete_message(channel_bot.as_str(), &message.id).await;
                            } else {
                                //let test1 = Address::from_str("1").unwrap();
                                //let test2 = test1.to_string()
                                //let test2 = String::from_utf8(test1).unwrap()
                                tokio::time::sleep(time::Duration::from_millis(10)).await;
                            }
                        }
                        //tmp code


                        //block content logs [NewOrderFilter { user: 0xfaa56b120b8de4597cf20eff21045a9883e82aad, base_token: "BTC", quote_token: "USDT", amount: 3, price: 4 }]

                    }
                }
            });
        });
        s.spawn(move |_| {
            loop {
                let mut arc_rsmq = arc_rsmq.clone();
                let orders: Vec<BookOrder> =
                    event_receiver.recv().expect("failed to recv columns");
                println!(
                    "[listen_blocks: receive] New order Event {:?},base token {:?}",
                    orders[0].id, orders[0].side
                );
                //TODO: matched order
                //update OrderBook
                let mut agg_trades = Vec::<LastTrade2>::new();
                let mut add_depth = AddBook2 {
                    asks: HashMap::<u64,u64>::new(),
                    bids: HashMap::<u64,u64>::new(),
                };
                //let orders = Vec::new();
                let mut db_orders = Vec::<OrderInfo>::new();
                for  (index,order) in orders.into_iter().enumerate() {
                    let side_str = format!("{:?}",order.side);
                    let mut db_order = OrderInfo::new(order.id.clone(),"BTC-USDT".to_string(),order.account.clone(),side_str,order.price.clone(),order.amount.clone());
                    info!("start match_order index {}",index);
                    let matched_amount = match_order(order, &mut agg_trades, &mut add_depth);
                    db_order.status = if narrow(matched_amount) == db_order.amount {
                        "full_filled".to_string()
                    }else if  matched_amount != 0 && narrow(matched_amount) < db_order.amount{
                        "partial_filled".to_string()
                    }else if matched_amount == 0{
                        "pending".to_string()
                    }else {
                        assert!(false);
                        "".to_string()
                    };
                    db_order.matched_amount = narrow(matched_amount);
                    db_order.available_amount = db_order.amount - narrow(matched_amount);
                    db_orders.push(db_order);
                    info!("finished match_order index {}",index);
                }

                insert_order(db_orders);
                //todo: sync data to psql


                info!("finished compute  agg_trades {:?},add_depth {:?}",agg_trades,add_depth);

                //tmp code
                /***
                let updateBook = AddBook {
                    asks: vec![(5000.123, -1.1), (6000.123, 1.1)],
                    bids: vec![(4000.123, -1.1), (3000.123, 1.1)],
                };

                //update new trade
                let mut updateTrade = Vec::<LastTrade>::new();
                updateTrade.push(LastTrade {
                    id: "BTC-USDT".to_string(),
                    price: 1000.0,
                    amount: 10.1,
                    taker_side: "buy".to_string(),
                    updated_at: 1644287259123,
                });
                //理论上一次撮合taker_side是一样的
                updateTrade.push(LastTrade {
                    id: "BTC-USDT".to_string(),
                    price: 1001.0,
                    amount: 20.2,
                    taker_side: "sell".to_string(),
                    updated_at: 1644287259123,
                });
                 */


                let channel_update_book = channel_update_book.clone();
                let channel_new_trade = channel_new_trade.clone();
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let json_str = serde_json::to_string(&add_depth).unwrap();
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
    listen_blocks().await;
    Ok(())
}
