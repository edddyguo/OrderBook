mod util;

extern crate ethers_contract_abigen;
extern crate rsmq_async;
extern crate rand;
extern crate num;

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
use std::ops::{Add, Range};
use std::str::FromStr;
use std::sync::{mpsc, Arc, RwLock};
use tokio::runtime::Runtime;
use tokio::time;
use rand::prelude::SliceRandom;
use util::MathOperation;
use rand::Rng;
use rand::distributions::uniform::SampleRange;

abigen!(
    SimpleContract,
    "../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

async fn new_order(price: f64, amount: f64) {
    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");
    let price_nano = (price * 100000000.0) as u64;
    let amount_nano = (amount * 100000000.0) as u64;

    let event = NewOrderFilter {
        user: Address::from_str("0xbc1Bd19FD1b220e989F8bF75645b9B7028Fc255B").unwrap(),
        base_token: "USDT".to_string(),
        quote_token: "BTC".to_string(),
        side: "buy".to_string(),
        amount: U256::from(price_nano) ,
        price: U256::from(amount_nano),
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
    let base_price = 40000.0f64;
    let base_amount = 1.0f64;

    loop {
        let mut rng = rand::thread_rng();
        let price_add: f64 = rng.gen_range(-1000.0..1000.0);
        let amount_add: f64 = rng.gen_range(-1.0..1.0);
        let price = (base_price + price_add).to_fix(8);
        let amount = (base_amount + amount_add).to_fix(8);
        println!("[newOrder]:price {},amount {}",price,amount);
        new_order(price,amount).await;
        tokio::time::sleep(time::Duration::from_millis(5000)).await;
    }

    Ok(())
}
