mod util;

extern crate ethers_contract_abigen;
extern crate num;
extern crate rand;
extern crate rsmq_async;
extern crate core;

use ethers::{prelude::*, types::U256};

//use ethers::providers::Ws;
use ethers_contract_abigen::Address;

use rsmq_async::{Rsmq, RsmqConnection};

use std::fs::File;
use std::io::BufReader;

use std::str::FromStr;
use log::{error, info, warn};

use tokio::time;
use crate::num::ToPrimitive;

use rand::Rng;
use crate::abi::Abi;
use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};
use std::ops::{Div, Mul};
use clap::{App, Arg};
use common::utils::math::MathOperation;
use common::env;
use chemix_chain::chemix::ChemixContractClient;

use common::types::order::{Side, Status as OrderStatus};
use common::types::trade::Status as TradeStatus;
use common::types::order::Side as OrderSide;
use common::types::order::Side::{Buy, Sell};

abigen!(
    SimpleContract,
    "../contract/ChemixMain.json",
    //"../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
async fn new_order(client: ChemixContractClient, base_token: &str, quote_token: &str, side: Side, price: f64, amount: f64) {
    loop {
        match client.new_order(side.clone(), base_token, quote_token, price, amount).await {
            Ok(_) => {
                break;
            }
            Err(error) => {
                if error.to_string().contains("underpriced") {
                    warn!("gas too low and try again");
                    tokio::time::sleep(time::Duration::from_millis(5000)).await;
                } else {
                    //tmp code
                    error!("{}",error);
                    unreachable!()
                }
            }
        }
    }
}

//讨论：取消不需要两个token，全局的index
async fn cancel_order(client: ChemixContractClient, base_token: &str, quote_token: &str, index: u32) {
    loop {
        match client.cancel_order(base_token, quote_token, index).await {
            Ok(_) => {
                break;
            }
            Err(error) => {
                if error.to_string().contains("underpriced") {
                    warn!("gas too low and try again");
                    tokio::time::sleep(time::Duration::from_millis(5000)).await;
                } else {
                    //tmp code
                    error!("{}",error);
                    unreachable!()
                }
            }
        }
    }
}

async fn auto_take_order(client: ChemixContractClient, base_token: &str, quote_token: &str) {
    let base_price = 50000.0f64;
    let base_amount = 1.0f64;
    loop {
        let mut rng = rand::thread_rng();
        let price_add: f64 = rng.gen_range(-2000.0..2000.0);
        let amount_add: f64 = rng.gen_range(-1.0..1.0);
        let side_random: u8 = rng.gen_range(0..=1);
        let side = match side_random {
            0 => Buy,
            _ => Sell,
        };

        let price = (base_price + price_add).to_fix(8);
        let amount = (base_amount + amount_add).to_fix(8);
        println!(
            "[newOrder]: side {} price {},amount {}",
            side.as_str(), price, amount
        );

        new_order(client.clone(), base_token, quote_token, side.clone(), price, amount).await;
        tokio::time::sleep(time::Duration::from_millis(1000)).await;
    }
}


//test1,0x613548d151E096131ece320542d19893C4B8c901
//let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
//test2,0x37BA121cdE7a0e24e483364185E80ceF655346DD
//let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
//test3,0xca9B361934fc7A7b07814D34423d665268111726
//let pri_key = "b0a09e85dad814ccc7231982401cca5accc3a46bc68349b403a7a129517cc266";
//tj
//let pri_key = "1f3bc7d273c179f0b73745d0599a15ece081837a9aa4ccb6351842fcad19fb95";
//local
//let pri_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
//todo: 手续费处理
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let matches = App::new("bot")
        .version("1.0")
        .about("Does awesome things")
        .arg(Arg::new("pri_key")
            .required(true)
            .index(1))
        .arg(Arg::new("base_token")
            .about("base token contract address")
            .required(true)
            .index(2))
        .arg(Arg::new("quote_token")
            .about("quote token contract address")
            .required(true)
            .index(3))
        .subcommand(
            App::new("buy")
                .arg(
                    Arg::new("price")
                        .about("token price")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("amount")
                        .required(true)
                        .index(2)
                )
        )
        .subcommand(
            App::new("sell")
                    .arg(
                    Arg::new("price")
                        .about("token price")
                        .required(true)
                        .index(1),
                    )
                    .arg(
                        Arg::new("amount")
                        .required(true)
                        .index(2)
                    )
        )
        .subcommand(
            App::new("cancel").arg(
                Arg::new("index")
                    .about("order index you want cancel")
                    .required(true)
                    .index(1),
            ),
        )
        .subcommand(App::new("auto"))
        .get_matches();
    let pri_key: &str = matches.value_of("pri_key").unwrap();
    let base_token: &str = matches.value_of("base_token").unwrap();
    let quote_token: &str = matches.value_of("quote_token").unwrap();

    let chemix_main_addr = env::CONF.chemix_main.to_owned().unwrap();
    let client = ChemixContractClient::new(pri_key, chemix_main_addr.to_str().unwrap());

    match matches.subcommand() {
        Some(("buy", sub_matches)) => {
            let price_str = sub_matches.value_of("price").unwrap();
            let price = price_str.to_string().parse::<f64>().unwrap();
            let amount_str = sub_matches.value_of("amount").unwrap();
            let amount = amount_str.to_string().parse::<f64>().unwrap();
            new_order(client, base_token, quote_token, OrderSide::Buy, price, amount).await;
        }
        Some(("sell", sub_matches)) => {
            let price_str = sub_matches.value_of("price").unwrap();
            let price = price_str.to_string().parse::<f64>().unwrap();
            let amount_str = sub_matches.value_of("amount").unwrap();
            let amount = amount_str.to_string().parse::<f64>().unwrap();
            new_order(client, base_token, quote_token, OrderSide::Sell, price, amount).await;
        }
        Some(("cancel", sub_matches)) => {
            let index_str = sub_matches.value_of("index").unwrap();
            let index = index_str.to_string().parse::<u32>().unwrap();
            cancel_order(client, base_token, quote_token, index).await;
        }
        Some(("auto", _)) => {
            //generate_all_token(output_path, component_path,token_canister);
            auto_take_order(client, base_token, quote_token).await;
        }
        _ => { panic!("subcommand not support") }
    }

    Ok(())
}
