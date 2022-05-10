use crate::{gen_settle_trades, TradeStatus};
use chemix_chain::chemix::vault::{Vault};
use chemix_chain::chemix::ChemixContractClient;
use chemix_chain::{gen_txid, send_raw_transaction};
use chemix_models::market::get_markets;
use chemix_models::order::OrderInfoPO;
use chemix_models::trade::{list_trades, update_trades, TradeFilter, TradeInfoPO, UpdateTrade};
use common::queue::QueueType;
use common::types::trade::AggTrade;
use common::utils::algorithm::u8_arr_from_str;
use common::utils::math::u256_to_f64;
use common::utils::time::{get_current_time, get_unix_time};
use common::types::trade;
use anyhow::Result;
use thiserror::Error;

use rsmq_async::{Rsmq, RsmqConnection};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chemix_chain::bsc::{transaction_at};

//todo: 补充其他错误的处理，以及解冻的通样的处理
#[derive(Error, Debug)]
pub enum SettlementError {
    /// 存在已经处理过的order index,一般为分叉导致的
    #[error("order index `{0}` is already processed,raw error `{1}`")]
    OrderIndexAlreadyProcessed(u32,String),
    ///
    #[error("unkown error,contract error `{0}`")]
    Other(String),
}

pub async fn send_launch_trade(
    vault_settel_client: Arc<RwLock<ChemixContractClient<Vault>>>,
    last_order: &OrderInfoPO,
    db_trades: Vec<TradeInfoPO>,
) -> Result<(),SettlementError>{
    let settle_trades = gen_settle_trades(db_trades.clone());
    info!("settle trades {:?} ", settle_trades);
    let now = get_current_time();
    let hash_data = u8_arr_from_str(last_order.hash_data.clone());
    info!(
        "settlement_trades trade={:?}_index={},hash={:?}",
        settle_trades, last_order.index, hash_data
    );
    // 先更新db，在进行广播，如果失败，在监控确认逻辑中，该结算会一直处于launched状态（实际没发出去），在8个区块的检查时效后，
    // 状态重置为matched，重新进行清算，如果先广播再清算的话，如果广播后宕机，还没来得及更新db，就会造成重复清算
    let receipt = vault_settel_client
        .read()
        .unwrap()
        .settlement_trades(last_order.index, hash_data, settle_trades.clone())
        .await;
    let txid = gen_txid(&receipt);
    info!("[test_txid]::local {}", txid);
    let mut trades = db_trades
        .iter()
        .map(|x| UpdateTrade {
            id: x.id.clone(),
            status: TradeStatus::Launched,
            block_height: 0,
            transaction_hash: txid.clone(),
            hash_data: last_order.hash_data.clone(),
            updated_at: &now,
        })
        .collect::<Vec<UpdateTrade>>();
    update_trades(&trades);

    let receipt = send_raw_transaction(receipt)
        .await.map_err(|x| {
            let err_string = x.to_string();
           if err_string.contains("index already processed") {
               SettlementError::OrderIndexAlreadyProcessed(last_order.index,err_string)
           }else {
               SettlementError::Other(err_string)
           }
    })?;
    let transaction_hash = format!("{:?}", receipt.transaction_hash);
    info!("[test_txid]::remote {}", transaction_hash);
    assert_eq!(txid, transaction_hash);
    let height = receipt
        .block_number
        .unwrap()
        .to_string()
        .parse::<u32>()
        .unwrap();
    for trade in trades.iter_mut() {
        trade.block_height = height;
    }
    update_trades(&trades);
    Ok(())
}

pub async fn deal_launched_trade(
    new_settlements: Vec<String>,
    arc_queue: &Arc<RwLock<Rsmq>>,
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
            launched_trdade.push(UpdateTrade {
                id: x.id.clone(),
                status: TradeStatus::Confirmed,
                block_height,
                transaction_hash: x.transaction_hash,
                hash_data: x.hash_data,
                updated_at: &now,
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
                        agg_trades.insert(x.market_id.clone(), vec![agg_trade]);
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

//检查宕机时还没广播出去的交易重新广播
pub async fn check_last_launch() {
    let last_batch_trade = list_trades(TradeFilter::Height(0u32));
    if last_batch_trade.is_empty() {
        info!("Histrory launch is ok");
        
    }else {
        //check txhash is exist
        let batch_group_by = last_batch_trade.group_by(
            |a,b| a.transaction_hash == b.transaction_hash
        );
        //理论上只有一个transaction处于这种状态
        assert_eq!(batch_group_by.into_iter().count(),1);
        //确认的更新为confirm已经高度，没有上链的重新去launche
        let now = get_current_time();
        let uptrade = match transaction_at(&last_batch_trade.first().unwrap().transaction_hash).await {
            Some(height) => {
                last_batch_trade.into_iter().map(|x| {
                    UpdateTrade{
                        id: x.id,
                        status: trade::Status::Confirmed,
                        block_height: height as u32,
                        transaction_hash: x.transaction_hash,
                        hash_data: x.hash_data,
                        updated_at: &now
                    }
                }).collect::<Vec<UpdateTrade>>()
            }
            None => {
                last_batch_trade.into_iter().map(|x| {
                    UpdateTrade{
                        id: x.id,
                        status: trade::Status::Matched,
                        block_height: 0,
                        transaction_hash: x.transaction_hash,
                        hash_data: x.hash_data,
                        updated_at: &now
                    }
                }).collect::<Vec<UpdateTrade>>()
            }
        };
        update_trades(&uptrade);
    }
}

pub fn check_invalid_settelment(last_process_height: u32){
    let invalid_settelment = list_trades(TradeFilter::NotConfirm(last_process_height));
    let now = get_current_time();
    let reseted_trades = invalid_settelment.iter().map(|x|{
        UpdateTrade{
            id: x.id.clone(),
            status: trade::Status::Matched,
            block_height: 0,
            transaction_hash: "".to_string(),
            hash_data: "".to_string(),
            updated_at: &now
        }
    }).collect::<Vec<UpdateTrade>>();
    update_trades(&reseted_trades);
}
