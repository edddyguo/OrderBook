mod queue;

use anyhow::Result;
use ethers::prelude::*;
use std::collections::HashMap;

//use ethers::providers::Ws;
use ethers_contract_abigen::parse_address;
use ethers_providers::{Http, Middleware, Provider, StreamExt};
use rsmq_async::{Rsmq, RsmqConnection, RsmqError};
use chemix_chain::chemix::{CancelOrderState2, ChemixContractClient, SettleValues2, SettleValues3, ThawBalances, ThawBalances2};
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


use std::sync::Mutex;
use std::sync::{mpsc, Arc, RwLock};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time;

use common::utils::time as chemix_time;
use chrono::prelude::*;
use clap::{App, Arg};

use chemix_models::order::{get_order, insert_order, list_available_orders, update_order, EngineOrder, OrderInfo, UpdateOrder, BookOrder, get_last_order};
use chemix_models::trade::{insert_trades, list_trades, TradeInfo, update_trade};
use common::utils::algorithm::{sha256, sha2562, u8_arr_from_str};
use common::utils::math::{narrow, MathOperation, u256_to_f64};
use common::utils::time::get_current_time;
use common::utils::time::time2unix;
use ethers_core::abi::ethereum_types::U64;
use ethers_core::types::BlockId::Hash;
use log::info;
use chemix_models::api::get_markets;
use chemix_models::order::IdOrIndex::{Id, Index};
//use common::env::CONF as ENV_CONF;
use common::env::CONF as ENV_CONF;
use crate::queue::Queue;

use common::types::order::Status as OrderStatus;
use common::types::trade::Status as TradeStatus;
use common::types::order::Side as OrderSide;
use common::types::thaw::Status as ThawStatus;



#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;


static BaseTokenDecimal: u32 = 18;
static QuoteTokenDecimal: u32 = 15;

use chemix_models::api::MarketInfo;
use chemix_models::thaws::{list_thaws, update_thaws1};
use common::types::order::Status::FullFilled;

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


fn gen_settle_trades(db_trades: Vec<TradeInfo>) -> Vec<SettleValues3>{
    //key: account,token_address,is_positive
    let mut base_settle_values: HashMap<(String,String,bool), U256> = HashMap::new();
    let mut quote_settle_values: HashMap<(String,String,bool), U256> = HashMap::new();

    let mut update_base_settle_values = |k: &(String, String,bool), v: &U256| {
        match base_settle_values.get_mut(&k) {
            None => {
                base_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(mut tmp1) => {
                tmp1 = &mut tmp1.add(v);
            }
        }
    };
    let mut update_quote_settle_values = |k: &(String, String,bool), v: &U256| {
        match quote_settle_values.get_mut(&k) {
            None => {
                quote_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(mut tmp1) => {
                tmp1 = &mut tmp1.add(v);
            }
        }
    };

    let token_base_decimal = U256::from(10u128).pow(U256::from(18u32));
    for trader in db_trades.clone() {
        let base_amount = trader.amount;
        let quote_amount = trader.amount * trader.price / token_base_decimal;
        let market = get_markets(&trader.market_id);

        match trader.taker_side {
            OrderSide::Buy => {
                update_base_settle_values(&(trader.taker.clone(),market.base_token_address.clone(), true), &base_amount);
                update_quote_settle_values(&(trader.taker,market.quote_token_address.clone(), false), &quote_amount);

                update_base_settle_values(&(trader.maker.clone(),market.base_token_address.clone(), false), &base_amount);
                update_quote_settle_values(&(trader.maker,market.quote_token_address.clone(), true), &quote_amount);
            }
            OrderSide::Sell => {
                update_base_settle_values(&(trader.taker.clone(), market.base_token_address.clone(),false), &base_amount);
                update_quote_settle_values(&(trader.taker, market.quote_token_address.clone(),true), &quote_amount);

                update_base_settle_values(&(trader.maker.clone(),market.base_token_address.clone(), true), &base_amount);
                update_quote_settle_values(&(trader.maker,market.quote_token_address.clone(), false), &quote_amount);
            }
        }
    }

    /***
    let settle_trades = base_settle_values.iter().zip(quote_settle_values.iter()).map(|(base, quote)| {
        SettleValues2 {
            user: Address::from_str(base.0.0.as_str()).unwrap(),
            positiveOrNegative1: base.0.1,
            incomeBaseToken: base.1.to_owned(),
            positiveOrNegative2: quote.0.1,
            incomeQuoteToken: quote.1.to_owned(),
        }
    }).collect::<Vec<SettleValues2>>();
     */
    let mut settle_trades = base_settle_values.iter().map(|(k,v)| {
        SettleValues3{
            user: Address::from_str(k.0.as_str()).unwrap(),
            token: Address::from_str(k.1.as_str()).unwrap(),
            isPositive: k.2,
            incomeTokenAmount: v.to_owned()
        }
    }).collect::<Vec<SettleValues3>>();

    let mut settle_trades_quote = quote_settle_values.iter().map(|(k,v)| {
        SettleValues3{
            user: Address::from_str(k.0.as_str()).unwrap(),
            token: Address::from_str(k.1.as_str()).unwrap(),
            isPositive: k.2,
            incomeTokenAmount: v.to_owned()
        }
    }).collect::<Vec<SettleValues3>>();

    settle_trades.append(&mut settle_trades_quote);
    settle_trades
}

async fn listen_blocks(mut queue: Queue) -> anyhow::Result<()> {
    let mut last_height: U64 = U64::from(200u64);
    let arc_queue = Arc::new(RwLock::new(queue));
    let arc_queue = arc_queue.clone();
    let arc_queue2 = arc_queue.clone();

    let chemix_storage = ENV_CONF.chemix_storage.to_owned().unwrap();
    //test1
    let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    //let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";


    let mut chemix_main_client = ChemixContractClient::new(pri_key, chemix_storage.to_str().unwrap());
    let chemix_main_client_arc = Arc::new(RwLock::new(chemix_main_client));
    let chemix_main_client_receiver = chemix_main_client_arc.clone();
    let thaw_client = chemix_main_client_arc.clone();
    let battel_client = chemix_main_client_arc.clone();
    let ws_url = ENV_CONF.chain_ws.to_owned().unwrap();
    let rpc_url = ENV_CONF.chain_rpc.to_owned().unwrap();

    let watcher = Node::<Ws>::new(ws_url.to_str().unwrap()).await;
    let provider_http = Node::<Http>::new(rpc_url.to_str().unwrap());


    rayon::scope(|s| {
        //监听所有的settle和thaw事件并更新确认状态
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                //todo 过滤所有的thaws和battle，更新confirm状态
                /***
                for cancel_order in cancel_orders {
                    let order = get_order(Index(cancel_order.order_index.as_u32())).unwrap();

                    let update_info = UpdateOrder {
                        id: order.id,
                        status: OrderStatus::Canceled,
                        available_amount: U256::from(0i32),
                        matched_amount: order.matched_amount,
                        canceled_amount: order.available_amount,
                        updated_at: get_current_time(),
                    };
                    //todo: 批量更新
                    update_order(&update_info);
                }

                 */
            });
        });
        //execute thaw balance
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    let pending_thaws = list_thaws(ThawStatus::Pending);
                    if pending_thaws.is_empty() {
                        info!("Have no thaws need deal,and wait 5 seconds for next check");
                        tokio::time::sleep(time::Duration::from_millis(5000)).await;
                        continue;
                    }else {
                        info!("{:#?}",pending_thaws);
                    }
                    //todo: num_pow
                    //todo: 可以汇总
                    let mut thaw_infos = Vec::new();
                    for pending_thaw in pending_thaws.clone() {
                        let market = get_markets(pending_thaw.market_id.as_str());

                        let token_base_decimal = U256::from(10u128).pow(U256::from(market.base_contract_decimal));
                        let (token_address, amount,decimal) = match pending_thaw.side {
                            OrderSide::Sell => {
                                info!("available_amount {}",pending_thaw.amount);
                                (market.base_token_address, pending_thaw.amount,market.base_contract_decimal)
                            }
                            OrderSide::Buy => {
                                info!("available_amount {},price {},thaw_amount {}",pending_thaw.amount,pending_thaw.price,pending_thaw.amount * pending_thaw.price / token_base_decimal);
                                (market.quote_token_address, pending_thaw.amount * pending_thaw.price / token_base_decimal,market.quote_contract_decimal)
                            }
                        };
                        thaw_infos.push(ThawBalances {
                            token: Address::from_str(token_address.as_str()).unwrap(),
                            from: pending_thaw.account,
                            decimal: decimal as u32,
                            amount,
                        });
                    }
                    info!("all thaw info {:?}",thaw_infos);
                    info!("start thaw balance");

                    //let thaw_res = chemix_main_client2.read().unwrap().thaw_balances(thaw_infos).await.unwrap().unwrap();
                    let mut receipt = Default::default();

                    let order_json = format!(
                        "{}{}",
                        serde_json::to_string(&thaw_infos).unwrap(), get_current_time());
                    let cancel_id = sha2562(order_json);
                    loop {
                        match thaw_client.read().unwrap().thaw_balances(thaw_infos.clone(),cancel_id).await {
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
                    // info!("finish thaw balance res:{:?}",thaw_res);
                    info!("finish thaw balance res:{:?}",receipt);
                    let txid = format!("{:?}",receipt.transaction_hash);
                    let height = receipt.block_number.unwrap().as_u32() as i32;
                    let mut cancel_id_str= "".to_string();
                    for item in cancel_id {
                        //fixme: 0>2x?
                        let tmp = format!("{:x}", item);
                        cancel_id_str += &tmp;
                    }
                    //todo: 批处理
                    //pub fn update_thaws1(order_id:&str,cancel_id: &str,tx_id: &str,block_height:i32,status: ThawStatus) {
                    for pending_thaw in pending_thaws {
                        update_thaws1(pending_thaw.order_id.as_str(),cancel_id_str.as_str(),txid.as_str(),height,ThawStatus::Launched);
                    }

                    //解冻推送前端更新余额
                    //todo: 自己推送解冻事件，深度在ws里计算？
                    let arc_queue = arc_queue2.clone();
                    let new_thaw_queue = arc_queue.read().unwrap().ThawOrder.clone();
                    info!("__0001");
                    let thaw_infos2 = thaw_infos.iter().map(|x| {
                        ThawBalances2 {
                            token: x.token,
                            from: x.from,
                            amount: u256_to_f64(x.amount,x.decimal),
                        }
                    }).collect::<Vec::<ThawBalances2>>();
                    let json_str = serde_json::to_string(&thaw_infos2).unwrap();
                    arc_queue.write().unwrap().client
                        .send_message(new_thaw_queue.as_str(), json_str, None)
                        .await
                        .expect("failed to send message");

                    //更新深度
                    /***
                    let new_update_book_queue = arc_queue.read().unwrap().UpdateBook.clone();
                    info!("__0001");
                    let json_str = serde_json::to_string(&agg_trades).unwrap();
                    arc_queue.write().unwrap().client
                        .send_message(new_update_book_queue.as_str(), json_str, None)
                        .await
                        .expect("failed to send message");
                    ***/



                }
            });
        });
        //execute matched trade
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    //market_orders的移除或者减少
                    let u256_zero = U256::from(0i32);
                    //fix: list_trades 增加过滤条件
                    let db_trades = list_trades(None, None,Some(TradeStatus::Matched), 10000);
                    if db_trades.is_empty() {
                        info!("Have no matched trade need launch,and wait 5 seconds for next check");
                        tokio::time::sleep(time::Duration::from_millis(5000)).await;
                        continue;
                    }
                    let last_order = get_last_order().unwrap();
                    error!("db_trades = {:?}",db_trades);

                    //todo: settle traders
                    //let mut settle_values: HashMap<String, EnigneSettleValues> = HashMap::new();
                    //key: account,is_positive value amount

                    let settle_trades = gen_settle_trades(db_trades.clone());
                    info!("settle_trades {:?} ",settle_trades);

                    //fixme:有revert
                    //todo：空数组不清算
                    let hash_data = u8_arr_from_str(last_order.hash_data);
                    info!("__0000");

                    let mut agg_trades = Vec::new();
                    if !settle_trades.is_empty() {
                        let chemix_main_client2 = chemix_main_client_receiver.clone();
                        let mut receipt = Default::default();

                            loop {
//                            match chemix_main_client2.read().unwrap().settlement_trades(MARKET.base_token_address.as_str(),MARKET.quote_token_address.as_str(),settle_trades.clone()).await {
                                info!("settlement_trades____ trade={:?}_index={},hash={:?}",settle_trades,last_order.index,hash_data);
                                match chemix_main_client2.read().unwrap().settlement_trades2(last_order.index,hash_data,settle_trades.clone()).await {
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
                        //todo: update confirm

                        let height = receipt.block_number.unwrap().to_string().parse::<u32>().unwrap();
                        let txid = receipt.transaction_hash.to_string();
                        for db_trade in db_trades.clone() {
                            update_trade(db_trade.id.as_str(),TradeStatus::Launched,height,txid.as_str());
                        }

                        agg_trades = db_trades.iter().map(|x| {
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
                    }

                    //todo:update_trade launched


                    info!("finished compute  agg_trades {:?}",agg_trades);


                    //let channel_update_book = channel_update_book.clone();
                    //let channel_new_trade = channel_new_trade.clone();
                    let arc_queue = arc_queue.clone();
                    let new_trade_queue = arc_queue.read().unwrap().NewTrade.clone();
                    info!("__0001");
                        if !agg_trades.is_empty() {
                            let json_str = serde_json::to_string(&agg_trades).unwrap();
                            arc_queue.write().unwrap().client
                                .send_message(new_trade_queue.as_str(), json_str, None)
                                .await
                                .expect("failed to send message");
                        }
                    info!("__0002");
                }
            });

        });
    });
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let queue = Queue::new().await;
    listen_blocks(queue).await;
    Ok(())
}