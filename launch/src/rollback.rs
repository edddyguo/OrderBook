use std::ops::Deref;
use std::sync::{Arc, RwLock};
use ethers_core::types::BlockId;
use chemix_chain::bsc::get_block;
use chemix_chain::chemix::ChemixContractClient;
use chemix_chain::chemix::vault::Vault;
use chemix_models::order::{delete_orders, insert_orders, list_orders, OrderFilter, OrderInfoPO, update_orders, UpdateOrder};
use chemix_models::trade::{delete_trades, insert_trades, list_trades, TradeFilter, TradeInfoPO};
use chemix_models::forked_trade::*;
use chemix_models::{transactin_begin, transactin_commit};
use chemix_models::forked_order::{delete_forked_orders, ForkedOrderFilter, insert_forked_orders, list_forked_orders};
use common::types::order::Status;
use common::utils::math::U256_ZERO;
use common::utils::time::get_current_time;
use crate::get_last_process_height;

///回滚market_order，删除taker_order,删除trade
fn rollback_settlement(mut trades: Vec<TradeInfoPO>) -> Vec<OrderInfoPO>{
    //fixme: 同一区块清算的顺序是否有影响？
    let now = get_current_time();
    let mut rollback_taker_orders = Vec::new();
    //由于marke_order和taker_order都有多次撮合的情况，这里每次回滚一个trade就开始落表
    for trade in &trades {
        let taker_orders = list_orders(OrderFilter::ById(&trade.taker_order_id)).unwrap();
        //因为一个taker_orders可以撮合多个marker_order，所以这里的delete有时候会为删空
        if !rollback_taker_orders.iter().any(|x : &OrderInfoPO | x.id == taker_orders[0].id){
            rollback_taker_orders.push(taker_orders[0].clone());
        }
        //fixme: 因为当前的限价机制，taker_order_id都能一次性结算
        //如果这里是回滚点下的订单，则该taker_order_id是处于此块里。在engine那边要从回滚点所在的区块开始扫描
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
        update_orders(&vec![new_market_order]);
        //根据下单高度判断
    }
    rollback_taker_orders
}

fn rollback_thaws(){
    todo!()
}

/***
1、删除当前高度的forked表中的trade数据
2、插入到chemix_trade
3、删除当前高度的forketd表中的order数据
4、插入到chemix_order
5、market_order进行对应的正向加减
*/
fn revert_settlement(height: u32){
    let now = get_current_time();
    let mut old_forked_trades  = list_forked_trades(ForkedTradeFilter::ByHeight(height));
    delete_forked_trades(ForkedTradeFilter::ByHeight(height));
    insert_trades(&mut old_forked_trades);

    for trade in old_forked_trades {
        let marker_orders = list_orders(OrderFilter::ById(&trade.maker_order_id)).unwrap();
        let mut marker_order= marker_orders.first().unwrap();
        let new_available =  marker_order.available_amount - trade.amount;
        assert!(marker_order.amount >= new_available);
        //正向回滚对应的amount和订单状态
        let new_match = marker_order.matched_amount + trade.amount;
        let new_status =  if marker_order.available_amount == U256_ZERO {
            Status::FullFilled
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
        update_orders(&vec![new_market_order]);
    }

    let mut old_forked_orders  = list_forked_orders(ForkedOrderFilter::ByHeight(height)).unwrap();
    delete_forked_orders(ForkedOrderFilter::ByHeight(height));
    insert_orders(&mut old_forked_orders);
}

fn revert_thaws(height: u32){
    todo!()
}

pub async fn rollback_history_trade(arc_client: Arc<RwLock<ChemixContractClient<Vault>>>){
    //从当前区块往历史区块查询，order_index，区块高度和区块内处理的index完全一致的视为分叉点,通过事件过滤
    let mut check_height = get_last_process_height().await;
    let current_height = check_height;
    let mut total_rollback_taker_orders = Vec::new();
    let mut total_rollback_trades = Vec::new();
    //回滚分叉的无效的交易，直到找到回滚点（区块中的清算和数据库一致）
    loop {
        let block_hash = get_block(BlockId::from(check_height as u64))
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
        let settlemnt_trade = list_trades(TradeFilter::Height(check_height));
        let local_settlement_flags = settlemnt_trade.iter().map(|x| {
            x.hash_data.to_owned()
        }).collect::<Vec<String>>();
        if chain_settlement_flags.eq(&local_settlement_flags) {
            //找到订单分叉点，从该分叉点之后重新开始扫描区块内的交易，通知engine开始走扫历史区块的逻辑
            info!("Rollback finished rollback height {}",check_height);
            break;
        }
        check_height -= 1;
        total_rollback_trades.append(&mut settlemnt_trade.clone());
        let mut delete_taker_orders = rollback_settlement(settlemnt_trade);
        total_rollback_taker_orders.append(&mut delete_taker_orders);
    }


    //从block里对比之前的临时表的数据，如果一致则说明分叉摆回来了，则将临时表数据挪回来，
    for  block_height in check_height+1.. current_height{
        let block_hash = get_block(BlockId::from(block_height as u64))
            .await
            .unwrap()
            .unwrap()
            .hash
            .unwrap();
        let chain_settlement_flags = arc_client
            .write()
            .unwrap()
            .filter_settlement_event(block_hash)
            .await
            .unwrap();
        let old_forked_trade = list_forked_trades(ForkedTradeFilter::ByHeight(block_height));
        let forked_settlement_flags = old_forked_trade.iter().map(|x| {
            x.hash_data.to_owned()
        }).collect::<Vec<String>>();
        if !forked_settlement_flags.is_empty() && chain_settlement_flags.eq(&forked_settlement_flags) {
            revert_settlement(block_height);
        }
    }
    //为了避免主键冲突，先删除原有的脏数据，脏数据插入forked表的操作放在最后
    insert_forked_trades(&mut total_rollback_trades);
    insert_forked_orders(&mut total_rollback_taker_orders);

}