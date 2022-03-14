#![feature(slice_group_by)]

use ethers::prelude::*;
use std::collections::{HashMap, HashSet};

//use ethers::providers::Ws;

use chemix_chain::chemix::{ChemixContractClient, ThawBalances2};
use common::queue::*;
use ethers_providers::StreamExt;
use rsmq_async::{Rsmq, RsmqConnection};

use chemix_chain::bsc::{get_block, get_current_block};
use std::string::String;

use serde::Serialize;

use ethers::types::Address;
use std::fmt::Debug;
use std::ops::{Add, Sub};
use std::str::FromStr;

use std::sync::{Arc, RwLock};

use tokio::runtime::Runtime;
use tokio::time;

use chemix_models::order::{list_orders, OrderFilter};
use chemix_models::trade::{
    list_trades, list_trades2, update_trade, update_trade_by_hash, TradeInfo,
};
use common::utils::algorithm::{sha256, u8_arr_from_str, u8_arr_to_str};
use common::utils::math::{u256_to_f64, U256_ZERO};
use common::utils::time::get_current_time;

use ethers_core::abi::ethereum_types::U64;

use chemix_chain::chemix::vault::{SettleValues3, ThawBalances, Vault};
use chemix_models::market::get_markets;
use log::info;

//use common::env::CONF as ENV_CONF;
use chemix_models::thaws::{list_thaws3, Thaws, ThawsFilter};
use common::env::CONF as ENV_CONF;

use common::types::order::{Side as OrderSide, Side};
use common::types::thaw::Status as ThawStatus;
use common::types::trade::Status as TradeStatus;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

static BaseTokenDecimal: u32 = 18;
static QuoteTokenDecimal: u32 = 15;

const CONFIRM_HEIGHT: u32 = 2;

use chemix_models::thaws::{update_thaws1};
use common::types::depth::{Depth, RawDepth};

#[derive(Clone, Serialize, Debug)]
pub struct EnigneSettleValues {
    pub incomeQuoteToken: I256,
    pub incomeBaseToken: I256,
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

fn gen_settle_trades(db_trades: Vec<TradeInfo>) -> Vec<SettleValues3> {
    //key: account,token_address,is_positive
    let mut base_settle_values: HashMap<(String, String, bool), U256> = HashMap::new();
    let mut quote_settle_values: HashMap<(String, String, bool), U256> = HashMap::new();

    let mut update_base_settle_values =
        |k: &(String, String, bool), v: &U256| match base_settle_values.get_mut(&k) {
            None => {
                base_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(tmp1) => {
                *tmp1 = tmp1.add(v);
            }
        };
    let mut update_quote_settle_values =
        |k: &(String, String, bool), v: &U256| match quote_settle_values.get_mut(&k) {
            None => {
                quote_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(tmp1) => {
                *tmp1 = tmp1.add(v);
            }
        };

    for trader in db_trades.clone() {
        let market = get_markets(&trader.market_id).unwrap();
        let token_base_decimal = teen_power!(market.base_contract_decimal);

        let base_amount = trader.amount;
        let quote_amount = trader.amount * trader.price / token_base_decimal;

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

fn update_depth(depth_ori: &mut RawDepth, x: &TradeInfo) {
    let amount = I256::try_from(x.amount).unwrap();
    //maker吃掉的部分都做减法
    match x.taker_side {
        Side::Buy => {
            match depth_ori.asks.get_mut(&x.price) {
                None => {
                    depth_ori.asks.insert(x.price, -amount);
                }
                Some(tmp1) => {
                    *tmp1 = tmp1.sub(amount);
                }
            };
        }
        Side::Sell => {
            match depth_ori.bids.get_mut(&x.price) {
                None => {
                    depth_ori.bids.insert(x.price, -amount);
                }
                Some(tmp1) => {
                    *tmp1 = tmp1.sub(amount);
                }
            };
        }
    }
}

fn gen_depth_from_trades(trades: Vec<TradeInfo>) -> HashMap<String, Depth> {
    let mut all_market_depth = HashMap::<String, RawDepth>::new();
    let iters = trades.group_by(|a, b| a.market_id == b.market_id);
    for iter in iters.into_iter() {
        //todo:底层封装

        let market_id = iter[0].market_id.clone();
        let mut market_AddBook2 = RawDepth {
            asks: HashMap::new(),
            bids: HashMap::new(),
        };

        let mut taker_order_ids = HashSet::<String>::new();
        //maker吃掉的部分都做减法
        for trade in iter {
            taker_order_ids.insert(trade.taker_order_id.clone());
            update_depth(&mut market_AddBook2, trade);
        }

        //taker剩余的部分为插入
        for taker_order_id in taker_order_ids {
            let taker_orders = list_orders(OrderFilter::ById(taker_order_id.clone())).unwrap();
            let taker_order = taker_orders.first().unwrap();
            let matched_trades = list_trades2(
                &taker_order_id,
                &taker_order.hash_data,
                TradeStatus::Launched,
            );
            info!("[test_big_taker]:0000_matched_trades_{:?}", matched_trades);
            let mut matched_amount = U256_ZERO;
            for matched_trade in matched_trades {
                matched_amount += matched_trade.amount;
            }

            let confirmed_trades = list_trades2(
                &taker_order_id,
                &taker_order.hash_data,
                TradeStatus::Confirmed,
            );
            //todo！判断当前taker_order_id confirm的数量，处理一个taker_order_id多次上链的情况
            //若大额吃单,会放多个区块打包，第一次的时候将已撮合还没上链的余额更新到taker_side的方向剩余，之后上链的在该vlomue上累减
            if confirmed_trades.is_empty() {
                info!(
                    "[test_big_taker]:0001_taker_order.amount {},matched_amount {}",
                    taker_order.amount, matched_amount
                );
                let remain = taker_order.amount - matched_amount;
                match taker_order.side {
                    Side::Buy => {
                        let stat = market_AddBook2
                            .bids
                            .entry(taker_order.price)
                            .or_insert(I256::from(0i32));
                        *stat += I256::from_raw(remain);
                    }
                    Side::Sell => {
                        let stat = market_AddBook2
                            .asks
                            .entry(taker_order.price)
                            .or_insert(I256::from(0i32));
                        *stat += I256::from_raw(remain);
                    }
                }
            } else {
                info!(
                    "[test_big_taker]:0002_taker_order.amount {},matched_amount {}",
                    taker_order.amount, matched_amount
                );
                match taker_order.side {
                    Side::Buy => {
                        let stat = market_AddBook2
                            .bids
                            .entry(taker_order.price)
                            .or_insert(I256::from(0i32));
                        *stat -= I256::from_raw(matched_amount);
                    }
                    Side::Sell => {
                        let stat = market_AddBook2
                            .asks
                            .entry(taker_order.price)
                            .or_insert(I256::from(0i32));
                        *stat -= I256::from_raw(matched_amount);
                    }
                }
            }
        }
        all_market_depth.insert(market_id, market_AddBook2);
    }

    let mut all_market_depth2 = HashMap::<String, Depth>::new();
    for (market_id, depth_raw) in all_market_depth.iter() {
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

        all_market_depth2.insert(
            market_id.to_string(),
            Depth {
                asks: asks2,
                bids: bids2,
            },
        );
    }
    all_market_depth2
}

fn gen_depth_from_thaws(pending_thaws: Vec<Thaws>) -> Depth {
    let mut add_depth = RawDepth {
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

    Depth {
        asks: asks2,
        bids: bids2,
    }
}

async fn deal_launched_trade(
    new_settlements: Vec<String>,
    arc_queue: Arc<RwLock<Rsmq>>,
    block_height: u32,
) {
    info!("Get settlement event {:?}", new_settlements);
    let mut agg_trades = HashMap::<String, Vec<LastTrade2>>::new();
    //目前来说一个区块里只有一个清算
    for hash_data in new_settlements {
        //todo: limit
        let db_trades = list_trades(
            None,
            None,
            Some(TradeStatus::Launched),
            Some(hash_data.clone()),
            Some(block_height),
            10000,
        );

        for x in db_trades.clone() {
            let user_price = u256_to_f64(x.price, QuoteTokenDecimal);
            let user_amount = u256_to_f64(x.amount, BaseTokenDecimal);
            if user_price != 0.0 && user_amount != 0.0 {
                match agg_trades.get_mut(x.market_id.as_str()) {
                    None => {
                        agg_trades.insert(
                            x.market_id.clone(),
                            vec![LastTrade2 {
                                id: x.id,
                                price: user_price,
                                amount: user_amount,
                                height: x.block_height,
                                taker_side: x.taker_side.clone(),
                            }],
                        );
                    }
                    Some(trades) => {
                        trades.push(LastTrade2 {
                            id: x.id,
                            price: user_price,
                            amount: user_amount,
                            height: x.block_height,
                            taker_side: x.taker_side.clone(),
                        });
                    }
                }
            }
        }

        update_trade_by_hash(TradeStatus::Confirmed, &hash_data, block_height);

        //push agg trade
        if !agg_trades.is_empty() {
            let json_str = serde_json::to_string(&agg_trades).unwrap();
            arc_queue
                .write()
                .unwrap()
                .send_message(QueueType::Trade.to_string().as_str(), json_str, None)
                .await
                .expect("failed to send message");
        }
    }
}

async fn deal_launched_thaws(new_thaw_flags: Vec<String>, arc_queue: Arc<RwLock<Rsmq>>) {
    for new_thaw_flag in new_thaw_flags {
        //推解冻信息
        ////flag足够，该flag在此时全部launched
        let pending_thaws = list_thaws3(ThawsFilter::ThawsHash(new_thaw_flag.clone()));
        let iters = pending_thaws.group_by(|a, b| a.market_id == b.market_id);

        for iter in iters.into_iter() {
            let mut thaw_infos = Vec::new();
            for pending_thaw in iter.clone() {
                update_thaws1(
                    pending_thaw.order_id.as_str(),
                    pending_thaw.thaws_hash.as_str(),
                    pending_thaw.transaction_hash.as_str(),
                    pending_thaw.block_height,
                    ThawStatus::Confirmed,
                );

                let market = get_markets(pending_thaw.market_id.as_str()).unwrap();
                let token_base_decimal = teen_power!(market.base_contract_decimal);
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
                            pending_thaw.amount * pending_thaw.price / token_base_decimal
                        );
                        (
                            market.quote_token_address,
                            pending_thaw.amount * pending_thaw.price / token_base_decimal,
                            market.quote_contract_decimal,
                        )
                    }
                };
                thaw_infos.push(ThawBalances {
                    token: Address::from_str(token_address.as_str()).unwrap(),
                    from: Address::from_str(pending_thaw.account.as_str()).unwrap(),
                    decimal: decimal as u32,
                    amount,
                });
            }

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
                .send_message(QueueType::Thaws.to_string().as_str(), json_str, None)
                .await
                .expect("failed to send message");
        }
    }
}

async fn listen_blocks(queue: Rsmq) -> anyhow::Result<()> {
    let _last_height: U64 = U64::from(200u64);
    let arc_queue = Arc::new(RwLock::new(queue));

    let pri_key = ENV_CONF.chemix_relayer_prikey.to_owned().unwrap();
    let chemix_vault_client =
        ChemixContractClient::<Vault>::new(pri_key.clone().to_str().unwrap());
    let chemix_vault_client = Arc::new(RwLock::new(chemix_vault_client));

    rayon::scope(|s| {
        let vault_listen_client = chemix_vault_client.clone();
        let vault_thaws_client = chemix_vault_client.clone();
        let vault_settel_client = chemix_vault_client.clone();

        //监听所有的settle事件并更新确认状态
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                //过滤所有的thaws和battle，更新confirm状态或者是未处理状态
                //fixme： 可以从第一个一个未处理的高度，如果都处理则从最后确认的高度开始，校验的时候判断是否已经是确认状态，防止重复确认
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
                            let new_settlements = vault_listen_client
                                .clone()
                                .write()
                                .unwrap()
                                .filter_settlement_event(block_hash.clone())
                                .await
                                .unwrap();
                            if new_settlements.is_empty() {
                                info!(
                                    "Not found settlement orders created at height {}",
                                    current_height
                                );
                            } else {
                                deal_launched_trade(new_settlements, arc_queue.clone(), height)
                                    .await;
                            }

                            let new_thaws = vault_listen_client
                                .clone()
                                .write()
                                .unwrap()
                                .filter_thaws_event(block_hash)
                                .await
                                .unwrap();
                            info!("new_orders_event {:?}", new_thaws);

                            if new_thaws.is_empty() {
                                info!(
                                    "Not found new order created at height {}",
                                    current_height
                                );
                            } else {
                                //只要拿到事件的hashdata就可以判断这个解冻是ok的，不需要比对其他
                                //todo： 另外起一个服务，循环判断是否有超8个区块还没确认的处理，有的话将起launch重新设置为pending
                                deal_launched_thaws(new_thaws, arc_queue.clone()).await;
                                //update状态为confirm
                            }
                        }
                    }

                    last_process_height = current_height - CONFIRM_HEIGHT;
                }
            });
        });
        //execute thaw balance
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    let pending_thaws = list_thaws3(ThawsFilter::Status(ThawStatus::Pending));
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
                        let market = get_markets(pending_thaw.market_id.as_str()).unwrap();
                        let token_base_decimal = teen_power!(market.base_contract_decimal);
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
                            from: Address::from_str(pending_thaw.account.as_str()).unwrap(),
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
                    let cancel_id = u8_arr_from_str(sha256(order_json));
                    loop {
                        match vault_thaws_client
                            .clone()
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
                    let transaction_hash = format!("{:?}", receipt.transaction_hash);
                    let height = receipt.block_number.unwrap().as_u32() as i32;
                    let cancel_id_str = u8_arr_to_str(cancel_id);
                    //todo: 批处理
                    //pub fn update_thaws1(order_id:&str,cancel_id: &str,tx_id: &str,block_height:i32,status: ThawStatus) {
                    for pending_thaw in pending_thaws.clone() {
                        update_thaws1(
                            pending_thaw.order_id.as_str(),
                            cancel_id_str.as_str(),
                            transaction_hash.as_str(),
                            height,
                            ThawStatus::Launched,
                        );
                    }
                }
            });
        });
        //execute matched trade
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                let mut last_height = get_current_block().await - 1;
                loop {
                    //market_orders的移除或者减少
                    //fix: 10000是经验值，放到外部参数注入
                    let db_trades = list_trades(None, None, Some(TradeStatus::Matched), None, None, 50);
                    if db_trades.is_empty() {
                        info!("Have no matched trade need launch,and wait 5 seconds for next check");
                        tokio::time::sleep(time::Duration::from_millis(5000)).await;
                        continue;
                    }
                    let last_orders = list_orders(OrderFilter::GetLastOne).unwrap();
                    let last_order = last_orders.first().unwrap();
                    error!("db_trades = {:?}",db_trades);

                    let settle_trades = gen_settle_trades(db_trades.clone());
                    info!("settle_trades {:?} ",settle_trades);

                    let hash_data = u8_arr_from_str(last_order.hash_data.clone());

                    //let mut agg_trades = Vec::new();
                    if !settle_trades.is_empty() {
                        while get_current_block().await - last_height == 0u32 {
                            info!("current {},wait for next block",last_height);
                            tokio::time::sleep(time::Duration::from_millis(500)).await;
                        }

                        let mut receipt = Default::default();
                        loop {
                            //                            match chemix_main_client2.read().unwrap().settlement_trades(MARKET.base_token_address.as_str(),MARKET.quote_token_address.as_str(),settle_trades.clone()).await {
                            info!("settlement_trades____ trade={:?}_index={},hash={:?}",settle_trades,last_order.index,hash_data);
                            match vault_settel_client.clone().read().unwrap().settlement_trades2(last_order.index, hash_data, settle_trades.clone()).await {
                                Ok(data) => {
                                    receipt = data.unwrap();
                                    last_height = receipt.block_number.unwrap().as_u32();
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

                        let height = receipt.block_number.unwrap().to_string().parse::<u32>().unwrap();
                        let transaction_hash = format!("{:?}", receipt.transaction_hash);
                        for db_trade in db_trades.clone() {
                            update_trade(db_trade.id.as_str(), TradeStatus::Launched, height, transaction_hash.as_str(), &last_order.hash_data);
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
    let queue = Queue::regist(vec![QueueType::Trade, QueueType::Depth, QueueType::Thaws]).await;
    listen_blocks(queue).await;
    Ok(())
}
