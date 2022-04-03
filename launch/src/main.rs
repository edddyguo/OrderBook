#![feature(slice_group_by)]

use ethers::prelude::*;
use std::cmp::max;
use std::collections::HashMap;

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
    list_trades, update_trade_by_hash, update_trades, TradeFilter, TradeInfoPO, UpdateTrade,
};
use common::utils::algorithm::{sha256, u8_arr_from_str, u8_arr_to_str};
use common::utils::math::u256_to_f64;
use common::utils::time::{get_current_time, get_unix_time};

use chemix_chain::chemix::vault::{SettleValues3, ThawBalances, Vault};
use chemix_models::market::get_markets;
use log::info;
use chemix_chain::{gen_txid, send_raw_transaction};

//use common::env::CONF as ENV_CONF;
use chemix_models::thaws::{list_thaws, ThawsFilter, UpdateThaw};
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

use chemix_models::thaws::update_thaws;
use common::types::depth::RawDepth;


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

fn gen_settle_trades(db_trades: Vec<TradeInfoPO>) -> Vec<SettleValues3> {
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
            is_positive: k.2,
            income_token_amount: v.to_owned(),
        })
        .collect::<Vec<SettleValues3>>();

    let mut settle_trades_quote = quote_settle_values
        .iter()
        .map(|(k, v)| SettleValues3 {
            user: Address::from_str(k.0.as_str()).unwrap(),
            token: Address::from_str(k.1.as_str()).unwrap(),
            is_positive: k.2,
            income_token_amount: v.to_owned(),
        })
        .collect::<Vec<SettleValues3>>();

    settle_trades.append(&mut settle_trades_quote);
    settle_trades
}

fn update_depth(depth_ori: &mut RawDepth, x: &TradeInfoPO) {
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
    let mut launched_trdade = Vec::new();
    let now = get_current_time();
    //目前来说一个区块里只有一个清算
    for hash_data in new_settlements {
        let db_trades = list_trades(TradeFilter::DelayConfirm(&hash_data, block_height));
        if db_trades.is_empty() {
            warn!(
                "This trade hash {} have already dealed,and jump it",
                hash_data.clone()
            );
            continue;
        }
        for x in db_trades {
            launched_trdade.push(UpdateTrade{
                id: x.id.clone(),
                status: TradeStatus::Confirmed,
                block_height,
                transaction_hash: x.transaction_hash,
                hash_data: x.hash_data,
                updated_at: &now
            });
            let market_info = get_markets(x.market_id.as_str()).unwrap();
            let base_token_decimal = market_info.base_contract_decimal;
            let quote_token_decimal = market_info.quote_contract_decimal;
            let user_price = u256_to_f64(x.price, quote_token_decimal);
            let user_amount = u256_to_f64(x.amount, base_token_decimal);
            if user_price != 0.0 && user_amount != 0.0 {
                let agg_trade = AggTrade {
                    id: x.id.clone(),
                    taker: x.taker.clone(),
                    maker: x.maker.clone(),
                    price: user_price,
                    amount: user_amount,
                    height: x.block_height,
                    taker_side: x.taker_side.clone(),
                    updated_at: get_unix_time(),
                };
                match agg_trades.get_mut(x.market_id.as_str()) {
                    None => {
                        agg_trades.insert(
                            x.market_id.clone(),
                            vec![agg_trade],
                        );
                    }
                    Some(trades) => {
                        trades.push(agg_trade);
                    }
                }
            }
        }

        //update_trade_by_hash(TradeStatus::Confirmed, &hash_data, block_height);
        update_trades(&launched_trdade);

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

async fn deal_launched_thaws(
    new_thaw_flags: Vec<String>,
    arc_queue: Arc<RwLock<Rsmq>>,
    height: u32,
) {
    let now = get_current_time();
    for new_thaw_flag in new_thaw_flags {
        //如果已经确认的跳过，可能发生在系统重启的时候
        let pending_thaws =
            list_thaws(ThawsFilter::DelayConfirm(&new_thaw_flag, height));
        if pending_thaws.is_empty() {
            warn!(
                "This thaw hash {} have already dealed,and jump it",
                new_thaw_flag.clone()
            );
            continue;
        }
        let iters = pending_thaws.group_by(|a, b| a.market_id == b.market_id);

        //所有交易对的解冻一起更新
        let mut update_thaws_arr = Vec::new();
        for iter in iters.into_iter() {
            let mut thaw_infos = Vec::new();
            for pending_thaw in iter.to_vec() {
                update_thaws_arr.push(UpdateThaw{
                    order_id: pending_thaw.order_id,
                    cancel_id: pending_thaw.thaws_hash.clone(),
                    block_height: height,
                    transaction_hash: pending_thaw.transaction_hash.clone(),
                    status: ThawStatus::Confirmed,
                    updated_at: &now
                });

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
        update_thaws(&update_thaws_arr);
    }
}

async fn get_last_process_height() -> u32 {
    let last_thaw = list_thaws(ThawsFilter::LastPushed);
    let last_trade = list_trades(TradeFilter::LastPushed);

    if last_thaw.len() == 0 && last_trade.len() == 0 {
        get_current_block().await
    } else if last_thaw.len() == 0 && last_trade.len() == 1 {
        last_trade[0].block_height as u32
    } else if last_thaw.len() == 1 && last_trade.len() == 0 {
        last_thaw[0].block_height as u32
    } else if last_thaw.len() == 1 && last_trade.len() == 1 {
        //因为解冻和清算同步扫块，所以这里取大数即可
        max(
            last_trade[0].block_height as u32,
            last_thaw[0].block_height as u32,
        )
    } else {
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
                info!("Start check history block from  {}", last_process_height);
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
                            info!("check height {}", height);
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
                                    height
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
                                    height
                                );
                            } else {
                                //只要拿到事件的hashdata就可以判断这个解冻是ok的，不需要比对其他
                                //todo： 另外起一个服务，循环判断是否有超8个区块还没确认的处理，有的话将起launch重新设置为pending
                                //但是什么场景下会出现没有被确认的情况？
                                deal_launched_thaws(new_thaws, arc_queue.clone(), height).await;
                            }
                        }
                        last_process_height = current_height - CONFIRM_HEIGHT;
                    }

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
                    info!("start thaw balance,all thaw info {:?}", thaw_infos);

                    let order_json = format!(
                        "{}{}",
                        serde_json::to_string(&thaw_infos).unwrap(),
                        get_current_time()
                    );
                    let cancel_id = u8_arr_from_str(sha256(order_json));

                    let raw_data =  vault_thaws_client
                            .clone()
                            .read()
                            .unwrap()
                            .thaw_balances(thaw_infos.clone(), cancel_id)
                            .await;
                    let txid = gen_txid(&raw_data);

                    let cancel_id_str = u8_arr_to_str(cancel_id);
                    let now = get_current_time();
                    let mut pending_thaws2 = pending_thaws.iter().map(|x| {
                        UpdateThaw {
                            order_id: x.order_id.clone(),
                            cancel_id: cancel_id_str.to_string(),
                            block_height: 0,
                            transaction_hash: txid.clone(),
                            status: ThawStatus::Launched,
                            updated_at: &now
                        }
                    }).collect::<Vec<UpdateThaw>>();
                    update_thaws(&pending_thaws2);
                    //todo: 此时节点问题或者分叉,或者gas不足
                    let receipt = send_raw_transaction(raw_data).await;
                    info!("finish thaw balance res:{:?}", receipt);
                    let transaction_hash = format!("{:?}", receipt.transaction_hash);
                    assert_eq!(txid,transaction_hash);
                    let height = receipt.block_number.unwrap().as_u32();
                    for pending_thaw in pending_thaws2.iter_mut() {
                        pending_thaw.block_height = height;
                    }
                    update_thaws(&pending_thaws2);
                    //todo: 取消的订单也超过100这种
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
                    info!("db_trades = {:?}",db_trades);

                    let settle_trades = gen_settle_trades(db_trades.clone());
                    info!("settle_trades {:?} ",settle_trades);

                    let hash_data = u8_arr_from_str(last_order.hash_data.clone());
                    if !settle_trades.is_empty() {
                        while get_current_block().await - last_height == 0u32 {
                            info!("current {},wait for next block",last_height);
                            tokio::time::sleep(time::Duration::from_millis(500)).await;
                        }
                        //todo: 先更新db，在进行广播，如果失败，在监控确认逻辑中，该结算会一直处于launched状态（实际没发出去），在8个区块的检查时效后，
                        // 状态重置为matched，重新进行清算，如果先广播再清算的话，如果广播后宕机，还没来得及更新db，就会造成重复清算
                        //todo: block_height为0的这部分交易放在新线程去处理
                        let now = get_current_time();
                        info!("settlement_trades trade={:?}_index={},hash={:?}",settle_trades,last_order.index,hash_data);
                        let mut receipt =  vault_settel_client
                            .clone()
                            .read()
                            .unwrap()
                            .settlement_trades2(last_order.index, hash_data, settle_trades.clone())
                            .await;
                        let txid = gen_txid(&receipt);
                        info!("[test_txid]::local {}",txid);
                        let mut trades = db_trades.iter().map(|x| {
                            UpdateTrade{
                                id: x.id.clone(),
                                status: TradeStatus::Launched,
                                block_height: 0,
                                transaction_hash: txid.clone(),
                                hash_data: last_order.hash_data.clone(),
                                updated_at: &now
                            }
                        }).collect::<Vec<UpdateTrade>>();
                        update_trades(&trades);

                        //todo: 此时节点问题或者分叉,待处理
                        let receipt = send_raw_transaction(receipt).await;
                        let transaction_hash = format!("{:?}", receipt.transaction_hash);
                        info!("[test_txid]::remote {}",transaction_hash);
                        assert_eq!(txid,transaction_hash);
                        let height = receipt.block_number.unwrap().to_string().parse::<u32>().unwrap();
                        for trade in trades.iter_mut() {
                            trade.block_height = height;
                        }
                        update_trades(&trades);
                        last_height = height;
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
