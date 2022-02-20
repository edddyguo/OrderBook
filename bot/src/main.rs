mod util;

extern crate ethers_contract_abigen;
extern crate num;
extern crate rand;
extern crate rsmq_async;

use ethers::{prelude::*, types::U256};

//use ethers::providers::Ws;
use ethers_contract_abigen::Address;

use rsmq_async::{Rsmq, RsmqConnection};

use std::env;
use std::fs::File;
use std::io::BufReader;

use std::str::FromStr;
use log::info;

use tokio::time;
use crate::num::ToPrimitive;

use rand::Rng;
use crate::abi::Abi;
use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};
use std::ops::{Div, Mul};
use chemix_utils::math::MathOperation;
use chemix_chain::chemix::ChemixContractClient;

abigen!(
    SimpleContract,
    "../contract/ChemixMain.json",
    //"../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);






async fn new_order(side: String, price: f64, amount: f64) {
    /***
    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");
    let price_nano = (price * 100000000.0) as u64;
    let amount_nano = (amount * 100000000.0) as u64;

    let event = NewOrderFilter {
        user: Address::from_str("0xbc1Bd19FD1b220e989F8bF75645b9B7028Fc255B").unwrap(),
        base_token: "USDT".to_string(),
        quote_token: "BTC".to_string(),
        side,
        amount: U256::from(amount_nano),
        price: U256::from(price_nano),
    };
    let events = vec![event];

    let json_str = serde_json::to_string(&events).unwrap();
    let channel_name = match env::var_os("CHEMIX_MODE") {
        None => "bot_local".to_string(),
        Some(mist_mode) => {
            format!("bot_{}", mist_mode.into_string().unwrap())
        }
    };

    rsmq.send_message(channel_name.as_str(), json_str, None)
        .await
        .expect("failed to send message");
    */
}

//fn cancle_order() {}

//todo: send bsc
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    env_logger::init();
    let base_price = 1001.0f64;
    let base_amount = 1.0f64;
    //get_dex_name().await;

    let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    let chemix_main_addr = "6a73e6c0a232C763dDe909bA6a92C92ed26B6ffa";
    let base_token = "18D5034280703EA96e36a50f6178E43565eaDc67";
    let quote_token = "7E62F80cA349DB398983E2Ee1434425f5B888f42";
    loop {
        let mut rng = rand::thread_rng();
        let price_add: f64 = rng.gen_range(-1000.0..1000.0);
        let amount_add: f64 = rng.gen_range(-1.0..1.0);
        let side_random: u8 = rng.gen_range(0..=1);
        let side = match side_random {
            0 => "buy",
            _ => "sell",
        };

        let price = (base_price + price_add).to_fix(8);
        let amount = (base_amount + amount_add).to_fix(8);
        println!(
            "[newOrder]: side {} price {},amount {}",
            side, price, amount
        );
        let client = ChemixContractClient::new(pri_key, chemix_main_addr);
        client.new_order(side,quote_token,base_token,price,amount).await.unwrap();
        tokio::time::sleep(time::Duration::from_millis(10000)).await;
    }
}
