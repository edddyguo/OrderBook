#![feature(slice_group_by)]

mod queue;

use ethers::prelude::*;
use std::collections::{HashMap, HashSet};

//use ethers::providers::Ws;

use chemix_chain::chemix::{ChemixContractClient, SettleValues3, ThawBalances, ThawBalances2};
use ethers_providers::{Http, StreamExt};
use rsmq_async::RsmqConnection;

use chemix_chain::bsc::Node;
use std::string::String;

use serde::Serialize;

use ethers::types::Address;
use std::fmt::Debug;
use std::ops::{Add, Sub};
use std::str::FromStr;

use std::sync::{Arc, RwLock};

use tokio::runtime::Runtime;
use tokio::time;

use chemix_models::order::{get_last_order, BookOrder, get_order, IdOrIndex};
use chemix_models::trade::{list_trades, update_trade, TradeInfo, update_trade_by_hash, list_trades2};
use common::utils::algorithm::{sha2562, u8_arr_from_str};
use common::utils::math::u256_to_f64;
use common::utils::time::get_current_time;

use ethers_core::abi::ethereum_types::U64;

use chemix_models::api::get_markets;
use log::info;

//use common::env::CONF as ENV_CONF;
use crate::queue::Queue;
use common::env::CONF as ENV_CONF;
use chemix_models::thaws::Thaws;

use common::types::order::{Side as OrderSide, Side};
use common::types::thaw::Status as ThawStatus;
use common::types::trade::{Status as TradeStatus, Status};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

static BaseTokenDecimal: u32 = 18;
static QuoteTokenDecimal: u32 = 15;

use chemix_models::thaws::{list_thaws, update_thaws1};

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

fn gen_settle_trades(db_trades: Vec<TradeInfo>) -> Vec<SettleValues3> {
    //key: account,token_address,is_positive
    let mut base_settle_values: HashMap<(String, String, bool), U256> = HashMap::new();
    let mut quote_settle_values: HashMap<(String, String, bool), U256> = HashMap::new();

    let mut update_base_settle_values =
        |k: &(String, String, bool), v: &U256| match base_settle_values.get_mut(&k) {
            None => {
                base_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(mut tmp1) => {
                *tmp1 = tmp1.add(v);
            }
        };
    let mut update_quote_settle_values =
        |k: &(String, String, bool), v: &U256| match quote_settle_values.get_mut(&k) {
            None => {
                quote_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(mut tmp1) => {
                *tmp1 = tmp1.add(v);
            }
        };

    let token_base_decimal = U256::from(10u128).pow(U256::from(18u32));
    for trader in db_trades.clone() {
        let base_amount = trader.amount;
        let quote_amount = trader.amount * trader.price / token_base_decimal;
        let market = get_markets(&trader.market_id);

        match trader.taker_side {
            OrderSide::Buy => {
                update_base_settle_values(
                    &(
                        trader.taker.clone(),
                        market.base_token_address.clone(),
                        true,
                    ),
                    &base_amount,
                );
                update_quote_settle_values(
                    &(trader.taker, market.quote_token_address.clone(), false),
                    &quote_amount,
                );

                update_base_settle_values(
                    &(
                        trader.maker.clone(),
                        market.base_token_address.clone(),
                        false,
                    ),
                    &base_amount,
                );
                update_quote_settle_values(
                    &(trader.maker, market.quote_token_address.clone(), true),
                    &quote_amount,
                );
            }
            OrderSide::Sell => {
                update_base_settle_values(
                    &(
                        trader.taker.clone(),
                        market.base_token_address.clone(),
                        false,
                    ),
                    &base_amount,
                );
                update_quote_settle_values(
                    &(trader.taker, market.quote_token_address.clone(), true),
                    &quote_amount,
                );

                update_base_settle_values(
                    &(
                        trader.maker.clone(),
                        market.base_token_address.clone(),
                        true,
                    ),
                    &base_amount,
                );
                update_quote_settle_values(
                    &(trader.maker, market.quote_token_address.clone(), false),
                    &quote_amount,
                );
            }
        }
    }

    let mut settle_trades = base_settle_values
        .iter()
        .map(|(k, v)| SettleValues3 {
            user: Address::from_str(k.0.as_str()).unwrap(),
            token: Address::from_str(k.1.as_str()).unwrap(),
            isPositive: k.2,
            incomeTokenAmount: v.to_owned(),
        })
        .collect::<Vec<SettleValues3>>();

    let mut settle_trades_quote = quote_settle_values
        .iter()
        .map(|(k, v)| SettleValues3 {
            user: Address::from_str(k.0.as_str()).unwrap(),
            token: Address::from_str(k.1.as_str()).unwrap(),
            isPositive: k.2,
            incomeTokenAmount: v.to_owned(),
        })
        .collect::<Vec<SettleValues3>>();

    settle_trades.append(&mut settle_trades_quote);
    settle_trades
}


fn update_depth(depth_ori: &mut AddBook2, x: &TradeInfo){
        let amount = I256::try_from(x.amount).unwrap();
        //maker吃掉的部分都做减法
        match x.taker_side {
            Side::Buy => {
                match depth_ori.asks.get_mut(&x.price) {
                    None => {
                        depth_ori.asks.insert(x.price, -amount);
                    }
                    Some(mut tmp1) => {
                        *tmp1 = tmp1.sub(amount);
                    }
                };
            },
            Side::Sell => {
                match depth_ori.bids.get_mut(&x.price) {
                    None => {
                        depth_ori.bids.insert(x.price, -amount);
                    }
                    Some(mut tmp1) => {
                        *tmp1 = tmp1.sub(amount);
                    }
                };
            }
        }
}

fn gen_depth_from_trades(trades: Vec<TradeInfo>) -> HashMap<String,AddBook> {
    let mut all_market_depth = HashMap::<String,AddBook2>::new();
    let mut iters = trades.group_by(|a,b| {
        a.market_id == b.market_id
    });
    for iter in iters.into_iter() {
        //todo:底层封装

        let market_id = iter[0].market_id.clone();
        let mut market_AddBook2 = AddBook2 {
            asks: HashMap::new(),
            bids: HashMap::new()
        };

        let mut taker_order_ids =  HashSet::<String>::new();
        //maker吃掉的部分都做减法
        for trade in iter{
            taker_order_ids.insert(trade.taker_order_id.clone());
            update_depth(&mut market_AddBook2,trade);
        }

        //taker剩余的部分为插入
        for taker_order_id in taker_order_ids {
            let taker_order = get_order(IdOrIndex::Id(taker_order_id.clone())).unwrap();
            let matched_trades = list_trades2(taker_order_id.clone(),taker_order.hash_data.clone());
            let mut matched_amount = U256::from(0);
            for matched_trade in matched_trades {
                matched_amount += matched_trade.amount;
            }

            let remain = taker_order.amount - matched_amount;
            match taker_order.side {
                Side::Buy => {
                    let stat = market_AddBook2
                        .bids
                        .entry(taker_order.price)
                        .or_insert(I256::from(0i32));
                    *stat += I256::from_raw(remain);
                },
                Side::Sell => {
                    let stat = market_AddBook2
                        .asks
                        .entry(taker_order.price)
                        .or_insert(I256::from(0i32));
                    *stat += I256::from_raw(remain);
                }
            }
        }
        all_market_depth.insert(market_id,market_AddBook2);
    }

    let mut all_market_depth2 = HashMap::<String,AddBook>::new();
    for (market_id,depth_raw) in all_market_depth.iter() {
        let asks2 = depth_raw
            .asks
            .iter()
            .map(|(x, y)| {
                let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
                // let user_volume = u256_to_f64(y.to_owned(), BaseTokenDecimal);
                let user_volume = if y < &I256::from(0u32) {
                    u256_to_f64(y.abs().into_raw(), BaseTokenDecimal) * -1.0f64
                } else {
                    u256_to_f64(y.abs().into_raw(), BaseTokenDecimal)
                };

                (user_price, user_volume)
            })
            .filter(|(p, v)| p != &0.0 && v != &0.0)
            .collect::<Vec<(f64, f64)>>();

        let bids2 = depth_raw
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

        all_market_depth2.insert(market_id.to_string(),AddBook {
            asks: asks2,
            bids: bids2,
        });
    }
    all_market_depth2
}


fn gen_depth_from_thaws(pending_thaws: Vec<Thaws>) -> AddBook{
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
            },
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

    info!("finished compute  ,add_depth {:?}", add_depth);
    let asks2 = add_depth
        .asks
        .iter()
        .map(|(x, y)| {
            let user_price = u256_to_f64(x.to_owned(), QuoteTokenDecimal);
            // let user_volume = u256_to_f64(y.to_owned(), BaseTokenDecimal);
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

async fn listen_blocks(queue: Queue) -> anyhow::Result<()> {
    let mut last_height: U64 = U64::from(200u64);
    let arc_queue = Arc::new(RwLock::new(queue));
    let arc_queue = arc_queue.clone();
    let arc_queue2 = arc_queue.clone();

    let chemix_vault = ENV_CONF.chemix_vault.to_owned().unwrap();
    //test1
    //let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    //let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
    let pri_key =  ENV_CONF.chemix_relayer_prikey.to_owned().unwrap();

    let chemix_main_client =
        ChemixContractClient::new(pri_key.to_str().unwrap(), chemix_vault.to_str().unwrap());
    let chemix_main_client_arc = Arc::new(RwLock::new(chemix_main_client));
    let chemix_main_client_receiver = chemix_main_client_arc.clone();
    let chemix_main_client_sender = chemix_main_client_arc.clone();
    let thaw_client = chemix_main_client_arc.clone();
    let _battel_client = chemix_main_client_arc.clone();
    let ws_url = ENV_CONF.chain_ws.to_owned().unwrap();
    let rpc_url = ENV_CONF.chain_rpc.to_owned().unwrap();

    let watcher = Node::<Ws>::new(ws_url.to_str().unwrap()).await;
    let provider_http = Node::<Http>::new(rpc_url.to_str().unwrap());

    rayon::scope(|s| {
        //监听所有的settle和thaw事件并更新确认状态
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                //todo 过滤所有的thaws和battle，更新confirm状态,并且推ws消息（解冻，depth、aggtrade）
                let mut stream = watcher.gen_watcher().await.unwrap();
                while let Some(block) = stream.next().await {
                    info!("block {}", block);
                    //last_height = last_height.add(1i32);
                    let current_height = provider_http.get_block(block).await.unwrap().unwrap();
                    //todo: 处理块高异常的情况,get block from http
                    //assert_eq!(last_height.add(1u64), current_height);
                    info!("new_orders_event {:?}", current_height);

                    let new_settlements = chemix_main_client_sender
                        .clone()
                        .write()
                        .unwrap()
                        .filter_settlement_event(current_height)
                        .await
                        .unwrap();
                    if new_settlements.is_empty() {
                        info!(
                            "Not found settlement orders created at height {}",
                            current_height
                        );
                    } else {
                        //let mut agg_trades = Vec::<LastTrade2>::new();
                        info!("Get settlement event {:?}",new_settlements);
                        let mut agg_trades = HashMap::<String,Vec<LastTrade2>>::new();
                        let mut add_depth = HashMap::<String,AddBook2>::new();
                        //目前来说一个区块里只有一个清算
                        for hash_data in new_settlements {
                            //todo: limit
                            let db_trades = list_trades(None,None,None,Some(hash_data.clone()),10000);
                            //todo: update status 可以根据状态一次update
                            update_trade_by_hash(TradeStatus::Confirmed,hash_data);
                            //todo: push ws aggTrade
                            //todo: push ws depth
                            for x in db_trades.clone(){
                                let user_price = u256_to_f64(x.price, QuoteTokenDecimal);
                                let user_amount = u256_to_f64(x.amount, BaseTokenDecimal);
                                if user_price != 0.0 && user_amount != 0.0 {
                                    match agg_trades.get_mut(x.market_id.as_str()) {
                                        None => {
                                            agg_trades.insert(x.market_id.clone(),vec![LastTrade2 {
                                                price: user_price,
                                                amount: user_amount,
                                                height: x.block_height as u32,
                                                taker_side: x.taker_side.clone(),
                                            }]);
                                        },
                                        Some(trades) => {
                                            trades.push(LastTrade2 {
                                                price: user_price,
                                                amount: user_amount,
                                                height: x.block_height as u32,
                                                taker_side: x.taker_side.clone(),
                                            });
                                        }
                                    }
                                }

                            }
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
                            let arc_queue = arc_queue.clone();
                            let new_depth_queue = arc_queue.read().unwrap().UpdateBook.clone();
                            let all_markets_depth = gen_depth_from_trades(db_trades.clone());
                            let json_str = serde_json::to_string(&all_markets_depth).unwrap();
                            arc_queue.write().unwrap().client
                                .send_message(new_depth_queue.as_str(), json_str, None)
                                .await
                                .expect("failed to send message");
                        }



                    }
                    /***
                    let new_orders = chemix_main_client_sender
                        .clone()
                        .write()
                        .unwrap()
                        .filter_new_order_event(current_height)
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

                     */
                    last_height = current_height;
                }
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
                    } else {
                        info!("{:#?}", pending_thaws);
                    }
                    //todo: num_pow
                    //todo: 可以汇总
                    let mut thaw_infos = Vec::new();
                    for pending_thaw in pending_thaws.clone() {
                        let market = get_markets(pending_thaw.market_id.as_str());

                        let token_base_decimal =
                            U256::from(10u128).pow(U256::from(market.base_contract_decimal));
                        let (token_address, amount, decimal) = match pending_thaw.side {
                            OrderSide::Sell => {
                                info!("available_amount {}", pending_thaw.amount);
                                (
                                    market.base_token_address,
                                    pending_thaw.amount,
                                    market.base_contract_decimal,
                                )
                            }
                            OrderSide::Buy => {
                                info!(
                                    "available_amount {},price {},thaw_amount {}",
                                    pending_thaw.amount,
                                    pending_thaw.price,
                                    pending_thaw.amount * pending_thaw.price
                                        / token_base_decimal
                                );
                                (
                                    market.quote_token_address,
                                    pending_thaw.amount * pending_thaw.price
                                        / token_base_decimal,
                                    market.quote_contract_decimal,
                                )
                            }
                        };
                        thaw_infos.push(ThawBalances {
                            token: Address::from_str(token_address.as_str()).unwrap(),
                            from: pending_thaw.account,
                            decimal: decimal as u32,
                            amount,
                        });
                    }
                    info!("all thaw info {:?}", thaw_infos);
                    info!("start thaw balance");

                    //let thaw_res = chemix_main_client2.read().unwrap().thaw_balances(thaw_infos).await.unwrap().unwrap();
                    let mut receipt = Default::default();

                    let order_json = format!(
                        "{}{}",
                        serde_json::to_string(&thaw_infos).unwrap(),
                        get_current_time()
                    );
                    let cancel_id = sha2562(order_json);
                    loop {
                        match thaw_client
                            .read()
                            .unwrap()
                            .thaw_balances(thaw_infos.clone(), cancel_id)
                            .await
                        {
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
                                    error!("{}", error);
                                    unreachable!()
                                }
                            }
                        }
                    }
                    // info!("finish thaw balance res:{:?}",thaw_res);
                    info!("finish thaw balance res:{:?}", receipt);
                    let txid = format!("{:?}", receipt.transaction_hash);
                    let height = receipt.block_number.unwrap().as_u32() as i32;
                    let mut cancel_id_str = "".to_string();
                    for item in cancel_id {
                        //fixme: 0>2x?
                        let tmp = format!("{:x}", item);
                        cancel_id_str += &tmp;
                    }
                    //todo: 批处理
                    //pub fn update_thaws1(order_id:&str,cancel_id: &str,tx_id: &str,block_height:i32,status: ThawStatus) {
                    for pending_thaw in pending_thaws.clone() {
                        update_thaws1(
                            pending_thaw.order_id.as_str(),
                            cancel_id_str.as_str(),
                            txid.as_str(),
                            height,
                            ThawStatus::Launched,
                        );
                    }

                    //解冻推送前端更新余额
                    //todo: 自己推送解冻事件，深度在ws里计算？
                    let arc_queue = arc_queue2.clone();
                    let new_thaw_queue = arc_queue.read().unwrap().ThawOrder.clone();
                    let thaw_infos2 = thaw_infos
                        .iter()
                        .map(|x| ThawBalances2 {
                            token: x.token,
                            from: x.from,
                            amount: u256_to_f64(x.amount, x.decimal),
                        })
                        .collect::<Vec<ThawBalances2>>();
                    let json_str = serde_json::to_string(&thaw_infos2).unwrap();
                    arc_queue
                        .write()
                        .unwrap()
                        .client
                        .send_message(new_thaw_queue.as_str(), json_str, None)
                        .await
                        .expect("failed to send message");

                    //更新深度
                    //todo: 是否放在confirm里面去推送
                    let add_depth = gen_depth_from_thaws(pending_thaws);
                    let new_update_book_queue = arc_queue.read().unwrap().UpdateBook.clone();
                    let json_str = serde_json::to_string(&add_depth).unwrap();
                    arc_queue.write().unwrap().client
                        .send_message(new_update_book_queue.as_str(), json_str, None)
                        .await
                        .expect("failed to send message");
                }
            });
        });
        //execute matched trade
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    //market_orders的移除或者减少
                    let _u256_zero = U256::from(0i32);
                    //fix: 10000是经验值，放到外部参数注入
                    let db_trades = list_trades(None, None,Some(TradeStatus::Matched), None,200);
                    if db_trades.is_empty() {
                        info!("Have no matched trade need launch,and wait 5 seconds for next check");
                        tokio::time::sleep(time::Duration::from_millis(5000)).await;
                        continue;
                    }
                    let last_order = get_last_order().unwrap();
                    error!("db_trades = {:?}",db_trades);

                    let settle_trades = gen_settle_trades(db_trades.clone());
                    info!("settle_trades {:?} ",settle_trades);

                    let hash_data = u8_arr_from_str(last_order.hash_data.clone());
                    info!("__0000");

                    //let mut agg_trades = Vec::new();
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
                            update_trade(db_trade.id.as_str(),TradeStatus::Launched,height,txid.as_str(),&last_order.hash_data);
                        }

                    }
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
