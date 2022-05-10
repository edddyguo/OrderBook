use std::sync::{Arc, RwLock};
use std::time;
use rsmq_async::{Rsmq, RsmqConnection};
use chemix_models::order::{list_orders, OrderFilter};
use common::queue::chain_status::ChainStatus;
use common::queue::QueueType;

pub async fn get_rollback_point(queue: &Arc<RwLock<Rsmq>>) -> Option<u32>{
    let mut index= 0;
    loop {
        match queue.write().unwrap()
            .receive_message::<String>(&QueueType::Chain.to_string(), Some(0u64))
            .await.unwrap() {
            None => {
                warn!("Status not initial");
                tokio::time::sleep(time::Duration::from_millis(1000)).await;
            }
            Some(message) => {
                match  ChainStatus::from(message.message.as_str()) {
                    ChainStatus::Stoped => {
                        panic!("Chain service not running");
                    }
                    ChainStatus::Forked => {
                        warn!("Wait Chain rollback finished");
                        tokio::time::sleep(time::Duration::from_millis(1000)).await;
                    }
                    ChainStatus::Healthy => {
                        info!("Chain is healthy");
                        break;
                    }
                }
            }
        }
        index += 1;
    }
    if index != 0 {
        let last_order = list_orders(OrderFilter::GetLastOne).unwrap();
        //只有存在历史订单的情况下，才会有订单分叉，这个unwrap即可
        Some(last_order.first().unwrap().block_height)
    }else {
        None
    }
}