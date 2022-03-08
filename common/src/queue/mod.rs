use rsmq_async::{Rsmq, RsmqConnection, RsmqError, RsmqOptions};
use std::env;
use std::ops::Deref;
use log::info;
use crate::env::CONF as ENV_CONF;

extern crate rustc_serialize;

use serde::Deserialize;

//#[derive(Serialize)]
use serde::Serialize;


lazy_static! {
    static ref CHEMIX_MODE: String = {
       let mode = ENV_CONF.chemix_mode.to_owned().unwrap();
        mode.to_str().unwrap().to_owned()
    };
    static ref REDIS_URL: String = {
        let redis = ENV_CONF.redis.to_owned().unwrap();
        redis.to_str().unwrap().to_owned()
    };
}

#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum QueueType {
    Thaws,
    Depth,
    Trade,
}
//update_book,new_trade_,thaw_order_
impl QueueType {
    pub fn to_string(&self) -> String {
        match self {
            /***
            Self::Thaws => "thaws",
            Self::Depth => "depth",
            Self::Trade => "trade",
             */
            Self::Thaws => format!("thaw_order_{}",*CHEMIX_MODE),
            Self::Depth => format!("update_book_{}",*CHEMIX_MODE),
            Self::Trade => format!("new_trade_{}",*CHEMIX_MODE),
        }
    }
}

pub struct Queue {}


impl Queue {
    async fn check_queue(client: &mut Rsmq, name: String) {
        let attributes = client.get_queue_attributes(name.to_owned().as_str()).await;
        match attributes {
            Ok(_) => {
                eprintln!("queue already exist");
            }
            Err(RsmqError::QueueNotFound) => {
                eprintln!("queue not found and create it");
                client.create_queue(name.as_str(), None, None, None)
                    .await
                    .expect("failed to create queue");
            }
            _ => {
                unreachable!()
            }
        }
    }

    pub async fn regist(types: Vec<QueueType>) -> Rsmq {
        let mut rsmq_option: RsmqOptions = Default::default();
        //todo: 不规则的redis连接处理
        let url_arr: Vec<&str>  = REDIS_URL.deref().split(&[':','/']).collect();
        rsmq_option.host = url_arr[3].to_owned();
        rsmq_option.port =  url_arr[4].to_owned();

        let mut rsmq = Rsmq::new(rsmq_option)
            .await
            .expect("redis connection failed");

        for queue_types in types {
            Queue::check_queue(&mut rsmq,queue_types.to_string()).await;
        }
        rsmq
    }
}