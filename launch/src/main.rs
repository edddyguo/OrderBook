#![feature(slice_group_by)]

mod thaw;
mod trade;

use ethers::prelude::*;
use std::cmp::max;
use std::collections::HashMap;

//use ethers::providers::Ws;

use chemix_chain::chemix::{ChemixContractClient, ThawBalances2};
use common::queue::*;
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
use chemix_models::trade::{list_trades, update_trades, TradeFilter, TradeInfoPO, UpdateTrade};
use common::utils::algorithm::{sha256, u8_arr_from_str, u8_arr_to_str};
use common::utils::math::u256_to_f64;
use common::utils::time::{get_current_time, get_unix_time};

use chemix_chain::chemix::vault::{SettleValues3, ThawBalances, Vault};
use chemix_chain::{gen_txid, send_raw_transaction};
use chemix_models::market::get_markets;
use log::info;

//use common::env::CONF as ENV_CONF;
use chemix_models::thaws::{list_thaws, ThawsFilter, UpdateThaw};
use common::env::CONF as ENV_CONF;

use common::types::order::{Side as OrderSide, Side};
use common::types::thaw::Status as ThawStatus;
use common::types::trade::{AggTrade, Status as TradeStatus};

extern crate lazy_static;

#[macro_use]
extern crate log;

#[macro_use]
extern crate common;

const CONFIRM_HEIGHT: u32 = 2;

use crate::thaw::{deal_launched_thaws, send_launch_thaw};
use crate::trade::{deal_launched_trade, send_launch_trade};
use chemix_models::thaws::update_thaws;
use common::types::depth::RawDepth;

#[derive(Clone, Serialize)]
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
        |k: &(String, String, bool), v: &U256| match base_settle_values.get_mut(k) {
            None => {
                base_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(tmp1) => {
                *tmp1 = tmp1.add(v);
            }
        };
    let mut update_quote_settle_values =
        |k: &(String, String, bool), v: &U256| match quote_settle_values.get_mut(k) {
            None => {
                quote_settle_values.insert(k.to_owned(), v.to_owned());
            }
            Some(tmp1) => {
                *tmp1 = tmp1.add(v);
            }
        };

    for trader in db_trades {
        let market = get_markets(&trader.market_id).unwrap();
        let token_base_decimal = u256_power!(10u32,market.base_contract_decimal);

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

fn _update_depth(depth_ori: &mut RawDepth, x: &TradeInfoPO) {
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

async fn get_last_process_height() -> u32 {
    let last_thaw = list_thaws(ThawsFilter::LastPushed);
    let last_trade = list_trades(TradeFilter::LastPushed);

    if last_thaw.is_empty() && last_trade.is_empty() {
        get_current_block().await
    } else if last_thaw.is_empty() && last_trade.len() == 1 {
        last_trade[0].block_height as u32
    } else if last_thaw.len() == 1 && last_trade.is_empty() {
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
    let chemix_vault_client = ChemixContractClient::<Vault>::new(pri_key.to_str().unwrap());
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
                                .filter_settlement_event(block_hash)
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
                                info!("Not found new thaws created at height {}", height);
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
                    send_launch_thaw(vault_thaws_client.clone(), pending_thaws).await;
                }
            });
        });
        //execute matched trade
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    //fix: 50是经验值，放到外部参数注入
                    //目前在engine模块保证大订单不再撮合
                    let db_trades = list_trades(TradeFilter::Status(TradeStatus::Matched,50));
                    //在撮合模块保证过大的单不进行撮合，视为非法订单,
                    //todo: 怎么获取500以内个数的，所有交易对的，所有账号的trade
                    assert!(db_trades.len() <= 500);
                    if db_trades.is_empty() {
                        info!("Have no matched trade need launch,and wait 5 seconds for next check");
                        tokio::time::sleep(time::Duration::from_millis(5000)).await;
                        continue;
                    }
                    let last_orders = list_orders(OrderFilter::GetLastOne).unwrap();
                    let last_order = last_orders.first().unwrap();
                    info!("db_trades = {:?}",db_trades);

                    //todo: block_height为0的这部分交易，只会在宕机的情况下出现，放在初始化的时候去处理
                    send_launch_trade(vault_settel_client.clone(),last_order,db_trades).await;

                }
            });
        });
    });
    Ok(())
}

//检查宕机时还没广播出去的交易，重新广播
async fn check_dirty_launch() {
    todo!()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    check_dirty_launch().await;
    let queue = Queue::regist(vec![QueueType::Trade, QueueType::Depth, QueueType::Thaws]).await;
    listen_blocks(queue).await
}
