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
use crate::abi::Abi;
use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};
use chemix_utils::math::MathOperation;

abigen!(
    SimpleContract,
    "../contract/ChemixMain.json",
    //"../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

static TokenADecimal : u8 = 3; //11 - 8
static TokenBDecimal : u8 = 13; //22 - 8




async fn new_order2 (side: &str, price: f64, amount: f64) -> String {
    //let host = "https://data-seed-prebsc-2-s3.binance.org:8545";
    let host = "http://192.168.1.158:8548";

    let provider_http = Provider::<Http>::try_from(host).unwrap();
    let wallet = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882"
        .parse::<LocalWallet>()
        .unwrap().with_chain_id(15u64);
    let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
    let client = Arc::new(client);
    //let contract_addr = Address::from_str("E41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C").unwrap();
    let contract_addr = Address::from_str("4CF5bd7EB82130763F8EdD0B8Ec44DFa21a5993e").unwrap();
    let contract = SimpleContract::new(contract_addr, client.clone());

    let amount = U256::from(TokenADecimal as u64 * amount.to_nano());
    let price = U256::from(TokenADecimal as u64 * price.to_nano());
    let quoteToken = Address::from_str("F20e4447DF5D02A9717a1c9a25B8d2FBF973bE56").unwrap();
    let baseToken = Address::from_str("A7A2a6A3D399e5AD69431aFB95dc86aff3BF871d").unwrap();

    match side {
        "buy" => {
            let result = contract.new_limit_buy_order(quoteToken,baseToken,price,amount)
                .legacy().send().await.unwrap().await.unwrap();
            info!("new buy order result  {:?}",result);
        },
        "sell" =>{
            let result = contract.new_limit_sell_order(quoteToken,baseToken,price,amount)
                .legacy().send().await.unwrap().await.unwrap();
            info!("new sell order result  {:?}",result);
        }
        _ => {}
    }
    "".to_string()
}

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
    let base_price = 40000.0f64;
    let base_amount = 1.0f64;
    //get_dex_name().await;

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
        new_order2(side, price, amount).await;
        tokio::time::sleep(time::Duration::from_millis(10000)).await;
    }

    //[newOrder]: side buy price 40503.19859207,amount 0.36172409
    // [newOrder]: side sell price 39036.04489557,amount 1.91700874
    //new_order("buy".to_string(),40503.19859207,0.36172409).await;
    //tokio::time::sleep(time::Duration::from_millis(5000)).await;
    //new_order("sell".to_string(),39036.04489557,1.91700874).await;

    Ok(())
}
