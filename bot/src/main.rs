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

use rand::Rng;
use util::MathOperation;
use crate::abi::Abi;
use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};

abigen!(
    SimpleContract,
    "../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);


async fn get_dex_name () -> String {
    //let host = "https://data-seed-prebsc-2-s3.binance.org:8545";
    let host = "http://192.168.1.158:8548";

    let provider_http = Provider::<Http>::try_from(host).unwrap();
    let wallet = "1b03a06c4a89d570a8f1d39e9ff0be8891f7657898675f11585aa7ec94fe2d12"
        .parse::<LocalWallet>()
        .unwrap().with_chain_id(15u64);
    let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
    let client = Arc::new(client);
    //let contract_addr = Address::from_str("E41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C").unwrap();
    let contract_addr = Address::from_str("AB1415967609bE6654a8e1FEDa209275DB1f5B9c").unwrap();

    let contract = SimpleContract::new(contract_addr, client.clone());
    let name = contract.dex_name().call().await.unwrap();
    info!("dex name {}",name);

    let amount = U256::from(3u64);
    let price = U256::from(4u64);
    let id = U256::from(5u64);
    let result = contract.new_order(id,"BTC".to_owned(),"USDT".to_owned(),"buy".to_owned(),amount,price)
        .legacy().send().await.unwrap().await.unwrap();

    info!("new order result  {:?}",result);

    name
}

async fn new_order(side: String, price: f64, amount: f64) {
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
}

//fn cancle_order() {}

//todo: send bsc
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    env_logger::init();
    let base_price = 40000.0f64;
    let base_amount = 1.0f64;
    get_dex_name().await;

    loop {
        break;
        let mut rng = rand::thread_rng();
        let price_add: f64 = rng.gen_range(-1000.0..1000.0);
        let amount_add: f64 = rng.gen_range(-1.0..1.0);
        let side_random: u8 = rng.gen_range(0..=1);
        let side = match side_random {
            0 => "buy".to_string(),
            _ => "sell".to_string(),
        };

        let price = (base_price + price_add).to_fix(8);
        let amount = (base_amount + amount_add).to_fix(8);
        println!(
            "[newOrder]: side {} price {},amount {}",
            side, price, amount
        );
        new_order(side, price, amount).await;
        tokio::time::sleep(time::Duration::from_millis(10000)).await;
    }

    //[newOrder]: side buy price 40503.19859207,amount 0.36172409
    // [newOrder]: side sell price 39036.04489557,amount 1.91700874
    //new_order("buy".to_string(),40503.19859207,0.36172409).await;
    //tokio::time::sleep(time::Duration::from_millis(5000)).await;
    //new_order("sell".to_string(),39036.04489557,1.91700874).await;

    Ok(())
}
