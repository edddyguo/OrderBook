pub mod order;

use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;

//use ethers::providers::Ws;

use chemix_chain::chemix::ChemixContractClient;
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use rsmq_async::{Rsmq, RsmqConnection};

use chemix_chain::bsc::{gen_watcher, get_block, get_last_block};
use std::string::String;

use serde::Serialize;
use std::convert::TryFrom;

use ethers::types::Address;
use std::fmt::Debug;
use std::ops::Sub;
use std::str::FromStr;

use crate::order::{cancel, gen_depth_from_order, match_order};
use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use std::time;

use tokio::runtime::Runtime;

use clap::{App, Arg};

use chemix_models::order::{get_order, insert_order, list_available_orders, update_order, BookOrder, EngineOrder, OrderInfo, UpdateOrder, get_last_order};
use chemix_models::trade::{insert_trades, TradeInfo};

use common::utils::math::u256_to_f64;
use common::utils::time::get_current_time;
use common::utils::time::time2unix;
use ethers_core::abi::ethereum_types::U64;
use chemix_chain::chemix::storage::Storage;
use chemix_chain::Node;

use chemix_models::order::IdOrIndex::{Id, Index};
use common::env::CONF as ENV_CONF;
use common::types::order::Status as OrderStatus;
use common::types::order::Side as OrderSide;
use chemix_models::market::{MarketInfo,get_markets};
use chemix_models::thaws::{insert_thaws, Thaws};
use common::queue::*;



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
        let available_sell : Vec<EngineOrder> = list_available_orders(MARKET.id.as_str(),OrderSide::Sell);
        let available_buy : Vec<EngineOrder> = list_available_orders(MARKET.id.as_str(),OrderSide::Buy);

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

async fn listen_blocks(queue: Rsmq) -> anyhow::Result<()> {
    //todo: 重启从上次结束的块开始扫
    info!("0000");
    let mut last_height: U64 = U64::from(200u64);
    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    let arc_queue = Arc::new(RwLock::new(queue));
    //不考虑安全性,随便写个私钥
    let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
    let chemix_storage_client = ChemixContractClient::<Storage>::new(pri_key);
    let chemix_storage_client = Arc::new(RwLock::new(chemix_storage_client));

    rayon::scope(|s| {
        //监听合约事件（新建订单和取消订单），将其发送到相应处理模块
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut watcher = gen_watcher().await;
                let mut stream = watcher.provide.watch_blocks().await.unwrap();
                let mut last_height = match  get_last_order() {
                    Some(order) => {
                        order.block_height
                    },
                    None => {
                        get_last_block().await
                    }
                };
                while let Some(block) = stream.next().await {
                    info!("current_book {:#?}", crate::BOOK.lock().unwrap());
                    let current_height = get_block(block).await.unwrap().unwrap().as_u32();
                    //防止ws推送的数据有跳空的情况
                    for height in last_height+1..=current_height {
                        info!("deal with block {:?},height {}", block,height);
                        //取消订单
                        let new_cancel_orders = chemix_storage_client
                            .clone()
                            .write()
                            .unwrap()
                            .filter_new_cancel_order_created_event(
                                height,
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
                                let order =
                                    get_order(Index(cancel_order.order_index.as_u32())).unwrap();
                                let update_info = UpdateOrder {
                                    id: order.id.clone(),
                                    status: OrderStatus::PreCanceled,
                                    available_amount: order.available_amount,
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

                        //过滤新下订单
                        let new_orders = chemix_storage_client
                            .clone()
                            .write()
                            .unwrap()
                            .filter_new_order_event(
                                height,
                                crate::MARKET.base_token_address.clone(),
                                crate::MARKET.quote_token_address.clone(),
                            )
                            .await
                            .unwrap();
                        info!("new_orders_event {:?}", new_orders);

                        if new_orders.is_empty() {
                            info!("Not found new order created at height {}", height);
                        } else {
                            let mut db_new_orders = Vec::new();
                            for order in new_orders {
                                let base_decimal = crate::MARKET.base_contract_decimal as u32;
                                let raw_amount = if base_decimal > order.num_power {
                                    order.amount * U256::from(10u32).pow(U256::from(base_decimal - order.num_power))
                                }else {
                                    order.amount / U256::from(10u32).pow(U256::from(order.num_power - base_decimal))
                                };
                                //todo: 非法数据过滤
                                db_new_orders.push(OrderInfo::new(
                                    order.id,  order.index, height, order.hash_data, crate::MARKET.id.to_string(),
                                    order.account, order.side, order.price,raw_amount));
                            }
                            event_sender
                                .send(db_new_orders)
                                .expect("failed to send orders");
                        }
                    }
                    last_height = current_height;
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

                let mut db_trades = Vec::<TradeInfo>::new();
                //market_orders的移除或者减少
                let mut db_marker_orders_reduce = HashMap::<String, U256>::new();
                let u256_zero = U256::from(0i32);

                for (index, db_order) in orders.iter_mut().enumerate() {
                    let _matched_amount = match_order(
                        db_order,
                        &mut db_trades,
                        &mut db_marker_orders_reduce,
                    );

                    info!(
                        "index {},taker amount {},matched-amount {}",
                        index, db_order.amount, _matched_amount
                    );
                    db_order.status = if db_order.available_amount == u256_zero{
                        OrderStatus::FullFilled
                    } else if db_order.available_amount != u256_zero && db_order.available_amount < db_order.amount {
                        OrderStatus::PartialFilled
                    } else if db_order.available_amount == db_order.amount {
                        OrderStatus::Pending
                    } else {
                        unreachable!()
                    };
                    //tmp code: 校验数据准确性用，后边移除
                    assert_eq!(_matched_amount,db_order.amount - db_order.available_amount);
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
                let u256_zero = U256::from(0i32);
                info!("db_marker_orders_reduce {:?}",db_marker_orders_reduce);
                for orders in db_marker_orders_reduce {
                    let marker_order_ori = get_order(Id(orders.0.clone())).unwrap();
                    let new_matched_amount = marker_order_ori.matched_amount + orders.1;
                    info!("marker_order_ori {};available_amount={},reduce_amount={}",marker_order_ori.id,marker_order_ori.available_amount,orders.1);
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

                //todo: 放在luanch模块在交易确认后推送？
                orders.retain(|x| x.matched_amount == U256::from(0u32));
                if !orders.is_empty() {
                    //todo：没有成交的里面推ws
                    let market_add_depth = gen_depth_from_order(orders);
                    let arc_queue = arc_queue.clone();
                    let rt = Runtime::new().unwrap();
                    rt.block_on(async move {
                        let json_str = serde_json::to_string(&market_add_depth).unwrap();
                        arc_queue
                            .write()
                            .unwrap()
                            .send_message(&QueueType::Depth.to_string(), json_str, None)
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
    let queue = Queue::regist(vec![QueueType::Depth]).await;
    info!("market {}", MARKET.base_token_address);
    info!("initial book {:#?}", crate::BOOK.lock().unwrap());
    listen_blocks(queue).await;
    Ok(())
}
