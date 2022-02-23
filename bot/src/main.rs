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
use log::{error, info, warn};

use tokio::time;
use crate::num::ToPrimitive;

use rand::Rng;
use crate::abi::Abi;
use std::{convert::TryFrom, path::Path, sync::Arc, time::Duration};
use std::ops::{Div, Mul};
use chemix_utils::math::MathOperation;
use chemix_chain::chemix::ChemixContractClient;
use chemix_models::order::Side::Sell;

abigen!(
    SimpleContract,
    "../contract/ChemixMain.json",
    //"../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

//fn cancle_order() {}

//todo: send bsc
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    env_logger::init();
    let base_price = 1001.0f64;
    let base_amount = 1.0f64;
    //get_dex_name().await;

    //test1
    let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    //test3
    //0xca9B361934fc7A7b07814D34423d665268111726
    //let pri_key = "b0a09e85dad814ccc7231982401cca5accc3a46bc68349b403a7a129517cc266";
    //tj
    //let pri_key = "1f3bc7d273c179f0b73745d0599a15ece081837a9aa4ccb6351842fcad19fb95";
    //local
   // let pri_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    /***
deployTokenA:   0xc739cD8920C65d372a0561507930aB6993c33E30
deployTokenB:   0x1982C0fC743078a7484bd82AC7A17BDab344308e
deployStorage:   0xAfC8a33002B274F43FC56D28D515406966354388
deployTokenProxy:   0x913e9d1a60bEb3312472A53CAe1fe64bC4df60e2
deployVault:   0x003fDe97E3a0932B2Bc709e952C6C9D73E0E9aE4
deployChemiMain:   0x0f48DDFe03827cd5Efb23122B44955c222eCd720

    */

    let chemix_main_addr = "0x0f48DDFe03827cd5Efb23122B44955c222eCd720";
    let base_token = "0xc739cD8920C65d372a0561507930aB6993c33E30"; //AAA 18
    let quote_token = "0x1982C0fC743078a7484bd82AC7A17BDab344308e"; //BBB 15

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
        //side sell price 142.21596998,amount 0.3266204
        // let price = 142.21596998;
        // let amount = 0.3266204;
        // let side = "sell";
        //todo: 手续费处理
        loop {
            match client.new_order(side,base_token,quote_token,price,amount).await {
                Ok(_) => {
                    break;
                }
                Err(error) => {
                    if error.to_string().contains("underpriced") {
                        warn!("gas too low and try again");
                        tokio::time::sleep(time::Duration::from_millis(5000)).await;
                    }else {
                        //tmp code
                        error!("{}",error);
                        unreachable!()
                    }
                }
            }
        }

        //client.new_order("sell",base_token,quote_token,1.0,1.0).await.unwrap();
        //client.new_order("buy",base_token,quote_token,1.0,1.0).await.unwrap();

        /***
        client.new_order("buy",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("buy",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("buy",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("buy",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("buy",quote_token,base_token,1.0,1.0).await.unwrap();

        client.new_order("sell",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("sell",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("sell",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("sell",quote_token,base_token,1.0,1.0).await.unwrap();
        client.new_order("sell",quote_token,base_token,1.0,1.0).await.unwrap();

        ***/
        tokio::time::sleep(time::Duration::from_millis(1000)).await;
    }
}
