use rsmq_async::{Rsmq, RsmqConnection, RsmqError};
use std::env;

pub struct Queue {
    pub client: Rsmq,
    pub NewTrade: String,
    pub UpdateBook: String,
    pub ThawOrder: String,
}

/***

async fn check_queue(name: &str) {
    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");

}

*/

impl Queue {
    async fn check_queue(name: String) {
        let mut rsmq = Rsmq::new(Default::default())
            .await
            .expect("connection failed");
        let attributes = rsmq.get_queue_attributes(name.to_owned().as_str()).await;
        match attributes {
            Ok(_) => {
                println!("queue already exist");
            }
            Err(RsmqError::QueueNotFound) => {
                println!("test2 not found");
                rsmq.create_queue(name.as_str(), None, None, None)
                    .await
                    .expect("failed to create queue");
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub async fn new() -> Queue {
        let rsmq = Rsmq::new(Default::default())
            .await
            .expect("connection failed");

        let channel_update_book = match env::var_os("CHEMIX_MODE") {
            None => "update_book_local".to_string(),
            Some(mist_mode) => {
                format!("update_book_{}", mist_mode.into_string().unwrap())
            }
        };

        let channel_new_trade = match env::var_os("CHEMIX_MODE") {
            None => "new_trade_local".to_string(),
            Some(mist_mode) => {
                format!("new_trade_{}", mist_mode.into_string().unwrap())
            }
        };

        let channel_thaw_order = match env::var_os("CHEMIX_MODE") {
            None => "thaw_order_local".to_string(),
            Some(mist_mode) => {
                format!("thaw_order_{}", mist_mode.into_string().unwrap())
            }
        };

        Queue::check_queue(channel_update_book.clone()).await;
        Queue::check_queue(channel_thaw_order.clone()).await;
        Queue::check_queue(channel_new_trade.clone()).await;

        Queue {
            client: rsmq,
            NewTrade: channel_new_trade,
            UpdateBook: channel_update_book,
            ThawOrder: channel_thaw_order,
        }
    }
}
