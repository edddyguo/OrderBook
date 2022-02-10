extern crate ethers_contract_abigen;
extern crate rsmq_async;

use anyhow::Result;
use ethers::{prelude::*, utils::Ganache,types::{U256}};
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_contract_abigen::{parse_address, Address};
use ethers_providers::{Http, Middleware, Provider, StreamExt, Ws};
use rsmq_async::{Rsmq, RsmqConnection, RsmqError, RsmqQueueAttributes};
use rustc_serialize::json;
use serde::Serialize;
use std::convert::TryFrom;
use std::ops::Add;
use std::str::FromStr;
use std::sync::{mpsc, Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::time;


abigen!(
    SimpleContract,
    "../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

async fn new_order() {
    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");

    let event = NewOrderFilter {
        user: Address::from_str("0xbc1Bd19FD1b220e989F8bF75645b9B7028Fc255B").unwrap(),
        base_token: "USDT".to_string(),
        quote_token: "BTC".to_string(),
        amount: U256::from(100) ,
        price: U256::from(100),
    };
    let events = vec![event];

    let json_str = serde_json::to_string(&events).unwrap();
    rsmq
        .send_message("bot", json_str, None)
        .await
        .expect("failed to send message");
}

//fn cancle_order() {}


//todo: send bsc
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    for _ in 0..10 {
        new_order().await;
    }
    Ok(())
}
