mod order;
mod trade;

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
use std::ops::Add;
use std::str::FromStr;
use std::sync::{mpsc, Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::time;
use std::sync::Mutex;
use crate::order::{BookOrder, EventOrder};

use chrono::offset::LocalResult;
use chrono::prelude::*;
use utils::{time as chemix_time,algorithm};
use ethers::{prelude::*};
use utils::math::MathOperation;
use ethers_core::abi::ethereum_types::{U256, U64};
use utils::algorithm::sha256;


#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;



#[derive(Clone, Serialize)]
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

#[derive(Debug, PartialEq, EthEvent)]
pub struct NewOrderEvent {
    user: Address,
    baseToken: String,
    quoteToken: String,
    amount: u64,
    price: u64,
}

#[derive(RustcEncodable, Clone, Serialize)]
pub struct AddBook {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}
#[derive(RustcEncodable, Clone, Serialize)]
pub struct MarketUpdateBook {
    id: String,
    data: AddBook,
}

#[derive(RustcEncodable, Clone, Serialize)]
pub struct LastTrade {
    id: String,
    price: f64,
    amount: f64,
    taker_side: String,
    updated_at: u64,
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
    check_queue("newTrade").await;
    check_queue("updateBook").await;


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
    let mut height = provider_http.get_block_number().await.unwrap();
    //166475590u64
    //16477780u64
    let mut height: U64 = U64::from(16647865u64);
    let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
    let client = Arc::new(client);

    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let mut arc_rsmq = Arc::new(RwLock::new(rsmq));
    let arc_rsmq2 = arc_rsmq.clone();
    rayon::scope(|s| {
        //send event in new block
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    dbg!(height);
                    let block_content = provider_http.get_block(height).await.unwrap();
                    if block_content.is_none() {
                        tokio::time::sleep(time::Duration::from_secs(2)).await;
                        println!("block not found,and wait a moment");
                    } else {
                        let addr = parse_address("0xE41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C")
                            .unwrap();
                        let contract = SimpleContract::new(addr, client.clone());
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
                        check_queue("bot").await;
                        let rsmq = arc_rsmq2.clone();
                        'listen_new_order : loop{
                            let message = rsmq.write().unwrap()
                                .receive_message::<String>("bot", None)
                                .await
                                .expect("cannot receive message");
                            if let Some(message) = message {
                                println!("receive new message {:?}", message.message);
                                let new_orders: Vec<NewOrderFilter> = serde_json::from_str(&message.message).unwrap();
                                println!("receive new order {:?} at {}", new_orders,chemix_time::get_current_time());
                                let new_orders = new_orders.iter().map(|x| {
                                    let now = Local::now().timestamp_millis() as u64;
                                    let order_json = format!("{}{}",serde_json::to_string(&x).unwrap(),now);
                                    let order_id = sha256(order_json);
                                    BookOrder {
                                        id: order_id,
                                        side: x.side.clone(),
                                        price: x.price.as_u64(),
                                        amount: x.amount.as_u64(),
                                        created_at: now,
                                    }
                                },).collect::<Vec<BookOrder>>();
                                event_sender.send(new_orders).expect("failed to send orders");
                                rsmq.write().unwrap().delete_message("bot", &message.id).await;
                            } else {
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
                let mut updateTrade2 = Vec::<LastTrade>::new();
                let mut updateBook2 = AddBook {
                    asks: vec![],
                    bids: vec![],
                };
                /**
                for  (index,order) in orders.into_iter().enumerate() {
                    match_order(order);
                }
                */



                //tmp code
                let updateBook = AddBook {
                    asks: vec![(1000.000, -10.0001), (2000.000, 10.0002)],
                    bids: vec![(1000.000, 10.0001), (2000.000, -10.0002)],
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

                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    let json_str = serde_json::to_string(&updateBook).unwrap();
                    arc_rsmq
                        .write()
                        .unwrap()
                        .send_message("updateBook", json_str, None)
                        .await
                        .expect("failed to send message");

                    let json_str = serde_json::to_string(&updateTrade).unwrap();
                    arc_rsmq
                        .write()
                        .unwrap()
                        .send_message("newTrade", json_str, None)
                        .await
                        .expect("failed to send message");
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
