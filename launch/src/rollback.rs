use std::sync::{Arc, RwLock};
use ethers_core::types::BlockId;
use chemix_chain::bsc::get_block;
use chemix_chain::chemix::ChemixContractClient;
use chemix_chain::chemix::vault::Vault;
use chemix_models::order::{delete_orders, list_orders, OrderFilter, update_orders, UpdateOrder};
use chemix_models::trade::{delete_trades, list_trades, TradeFilter, TradeInfoPO};
use chemix_models::invalied_trade::*;
use chemix_models::{transactin_begin, transactin_commit};
use common::types::order::Status;
use common::utils::time::get_current_time;
use crate::get_last_process_height;

fn rollback_settlement(mut trades: Vec<TradeInfoPO>){
    //fixme: 同一区块清算的顺序是否有影响？
    let now = get_current_time();
    let mut new_update_marke_orders = Vec::new();
    transactin_begin();
    for trade in &trades {
        //taker_order 和 对应的trade均无效删除，挪到临时表
        delete_orders(OrderFilter::ById(&trade.taker_order_id));
        delete_trades(TradeFilter::ById(&trade.id));
        let marker_orders = list_orders(OrderFilter::ById(&trade.maker_order_id)).unwrap();
        let mut marker_order= marker_orders.first().unwrap();
        let new_available =  marker_order.available_amount + trade.amount;
        assert!(marker_order.amount >= new_available);

        //回滚对应的amount和订单状态
        let new_match = marker_order.matched_amount - trade.amount;
        let new_status =  if marker_order.available_amount == marker_order.amount {
            Status::Pending
        }else {
            Status::PartialFilled
        };

        let new_market_order  = UpdateOrder {
            id: trade.maker_order_id.clone(),
            status: new_status,
            available_amount: new_available,
            canceled_amount: marker_order.canceled_amount,
            matched_amount: new_match,
            updated_at: &now
        };
        new_update_marke_orders.push(new_market_order);
    }

    update_orders(&new_update_marke_orders);
    insert_invalid_trades(&mut trades);
    transactin_commit();
}

fn rollback_thaws(){
    todo!()
}

pub async fn rollback_history_trade(arc_client: Arc<RwLock<ChemixContractClient<Vault>>>){
    //1、从当前区块往历史区块查询，order_index，区块高度和区块内处理的index完全一致的视为分叉点,通过事件过滤
    let mut block_height = get_last_process_height().await;
    loop {
        let block_hash = get_block(BlockId::from(block_height as u64))
            .await
            .unwrap()
            .unwrap()
            .hash
            .unwrap();
        //todo: 当前只有一个order index,待合约加上所有order_index 清算的逻辑
        let chain_settlement_flags = arc_client
            .write()
            .unwrap()
            .filter_settlement_event(block_hash)
            .await
            .unwrap();
        let settlemnt_trade = list_trades(TradeFilter::Height(block_height));
        let local_settlement_flags = settlemnt_trade.iter().map(|x| {
            x.hash_data.to_owned()
        }).collect::<Vec<String>>();

        //通过判断区块内的order_hash 是否一致，来判断订单是否分叉
        if chain_settlement_flags.eq(&local_settlement_flags){
            //找到订单分叉点，从改分叉点之后重新开始扫描区块内的交易，通知engine开始走扫历史区块的逻辑
            //todo: 本地还是通知engine端进行重新撮合？
            break;
        }else {
            rollback_settlement(settlemnt_trade);
            rollback_thaws();
        }

    }
}