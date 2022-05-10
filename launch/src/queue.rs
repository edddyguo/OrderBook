use rsmq_async::{Rsmq, RsmqConnection};
use common::queue::{Queue, QueueType};
use common::queue::chain_status::ChainStatus;

pub async fn init() -> Rsmq{
    let mut queue = Queue::regist(vec![QueueType::Trade, QueueType::Depth, QueueType::Thaws,QueueType::Chain]).await;
    update_chain_status(&mut queue,ChainStatus::Healthy).await;
    queue
}

pub async fn update_chain_status(queue:&mut Rsmq, value: ChainStatus) {
    let _res = queue.pop_message::<String>(&QueueType::Chain.to_string()).await.expect("queue pop message failed");
    queue.send_message(&QueueType::Chain.to_string(),value.as_str(), None)
        .await
        .expect("failed to send message");
}