#![feature(slice_group_by)]

use std::cmp::{max, min};
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
use chemix_models::trade::{list_trades, update_trade_by_hash, TradeInfo, TradeFilter, UpdateTrade, update_trades};
use common::utils::algorithm::{sha256, u8_arr_from_str, u8_arr_to_str};
use common::utils::math::{u256_to_f64, U256_ZERO};
use common::utils::time::{get_current_time, get_unix_time};

use ethers_core::abi::ethereum_types::U64;

use chemix_chain::chemix::vault::{SettleValues3, ThawBalances, Vault};
use chemix_models::market::get_markets;
use log::info;

//use common::env::CONF as ENV_CONF;
use chemix_models::thaws::{list_thaws, Thaws, ThawsFilter};
use common::env::CONF as ENV_CONF;

use common::types::order::{Side as OrderSide, Side};
use common::types::thaw::Status as ThawStatus;
use common::types::trade::{AggTrade, Status as TradeStatus};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

const CONFIRM_HEIGHT: u32 = 2;

use chemix_models::thaws::{update_thaws};
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


async fn deal_launched_trade(
    new_settlements: Vec<String>,
    arc_queue: Arc<RwLock<Rsmq>>,
    block_height: u32,
) {
    info!("Get settlement event {:?}", new_settlements);
    let mut agg_trades = HashMap::<String, Vec<AggTrade>>::new();
    //目前来说一个区块里只有一个清算
    for hash_data in new_settlements {
        let db_trades = list_trades(TradeFilter::DelayConfirm(hash_data.clone(),block_height));
        if db_trades.is_empty() {
            warn!("This trade hash {} have already dealed,and jump it",hash_data.clone());
            continue;
        }
        for x in db_trades.clone() {
            let market_info = get_markets(x.market_id.as_str()).unwrap();
            let base_token_decimal = market_info.base_contract_decimal;
            let quote_token_decimal = market_info.quote_contract_decimal;
            let user_price = u256_to_f64(x.price, quote_token_decimal);
            let user_amount = u256_to_f64(x.amount, base_token_decimal);
            if user_price != 0.0 && user_amount != 0.0 {
                match agg_trades.get_mut(x.market_id.as_str()) {
                    None => {
                        agg_trades.insert(
                            x.market_id.clone(),
                            vec![AggTrade {
                                id: x.id,
                                taker: x.taker.clone(),
                                maker: x.maker.clone(),
                                price: user_price,
                                amount: user_amount,
                                height: x.block_height,
                                taker_side: x.taker_side.clone(),
                                updated_at: get_unix_time(),
                            }],
                        );
                    }
                    Some(trades) => {
                        trades.push(AggTrade {
                            id: x.id,
                            taker: x.taker.clone(),
                            maker: x.maker.clone(),
                            price: user_price,
                            amount: user_amount,
                            height: x.block_height,
                            taker_side: x.taker_side.clone(),
                            updated_at: get_unix_time(),
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

async fn deal_launched_thaws(new_thaw_flags: Vec<String>, arc_queue: Arc<RwLock<Rsmq>>,height: u32) {
    for new_thaw_flag in new_thaw_flags {
        //如果已经确认的跳过，可能发生在系统重启的时候
        let pending_thaws = list_thaws(ThawsFilter::DelayConfirm(new_thaw_flag.clone(), height));
        if pending_thaws.is_empty() {
            warn!("This thaw hash {} have already dealed,and jump it",new_thaw_flag.clone());
            continue
        }
        let iters = pending_thaws.group_by(|a, b| a.market_id == b.market_id);

        for iter in iters.into_iter() {
            let mut thaw_infos = Vec::new();
            for pending_thaw in iter.clone() {
                update_thaws(
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

async fn get_last_process_height() -> u32{
    let last_thaw = list_thaws(ThawsFilter::LastPushed);
    let last_trade = list_trades(TradeFilter::LastPushed);

    if last_thaw.len() == 0 && last_trade.len() == 0{
        get_current_block().await
    } else if last_thaw.len() == 0 && last_trade.len() == 1 {
        last_trade[0].block_height as u32
    }else if last_thaw.len() == 1 && last_trade.len() == 0{
        last_thaw[0].block_height as u32
    }else if last_thaw.len() == 1 && last_trade.len() == 1{
        //因为解冻和清算同步扫块，所以这里取大数即可
        max(last_trade[0].block_height as u32, last_thaw[0].block_height as u32)
    }else {
        unreachable!()
    }
}

async fn listen_blocks(queue: Rsmq) -> anyhow::Result<()> {
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
                let mut last_process_height = get_last_process_height().await;
                info!("Start check history block from  {}",last_process_height);
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
                            info!("check height {}",height);
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
                                    "Not found new thaws created at height {}",
                                    current_height
                                );
                            } else {
                                //只要拿到事件的hashdata就可以判断这个解冻是ok的，不需要比对其他
                                //todo： 另外起一个服务，循环判断是否有超8个区块还没确认的处理，有的话将起launch重新设置为pending
                                deal_launched_thaws(new_thaws, arc_queue.clone(),height).await;
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
                    let pending_thaws = list_thaws(ThawsFilter::Status(ThawStatus::Pending));
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
                        update_thaws(
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
                    //fix: 50是经验值，放到外部参数注入
                    let db_trades = list_trades(TradeFilter::Status(TradeStatus::Matched,50));
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
                        //todo: 先更新db，在进行广播，如果失败，在监控确认逻辑中，该结算会一直处于launched状态（实际没发出去），在8个区块的检查时效后，
                        // 状态重置为matched，重新进行清算，如果先广播再清算的话，如果广播后宕机，还没来得及更新db，就会造成重复清算
                        let mut receipt = Default::default();
                        loop {
                            //match chemix_main_client2.read().unwrap().settlement_trades(MARKET.base_token_address.as_str(),MARKET.quote_token_address.as_str(),settle_trades.clone()).await {
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
                        //todo： 批量处理
                        let now = get_current_time();
                        let trades = db_trades.iter().map(|x| {
                            UpdateTrade{
                                id: x.id.clone(),
                                status: TradeStatus::Launched,
                                block_height: height,
                                transaction_hash:transaction_hash.clone(),
                                hash_data: last_order.hash_data.clone(),
                                updated_at: now.clone()
                            }
                        }).collect::<Vec<UpdateTrade>>();
                        update_trades(&trades);
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
