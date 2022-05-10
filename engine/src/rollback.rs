use std::sync::{Arc, RwLock};
use std::time;
use rsmq_async::{Rsmq, RsmqConnection};
use common::queue::chain_status::ChainStatus;
use common::queue::QueueType;

pub async fn check_chain_status(queue: &Arc<RwLock<Rsmq>>){
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
                        panic!("chain service not running");
                    }
                    ChainStatus::Forked => {
                        warn!("Wait Chain rollback finished");
                        tokio::time::sleep(time::Duration::from_millis(1000)).await;
                    }
                    ChainStatus::Healthy => {
                        info!("chain is healthy");
                        break;
                    }
                }

            }
        }
    }
}