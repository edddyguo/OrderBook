mod util;

extern crate ethers_contract_abigen;
extern crate num;
extern crate rand;
extern crate rsmq_async;

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
use chemix_utils::math::MathOperation;
use chemix_utils::env;
use chemix_chain::chemix::ChemixContractClient;
use chemix_models::order::Side::Sell;

abigen!(
    SimpleContract,
    "../contract/ChemixMain.json",
    //"../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

//fn cancle_order() {}

/***

  let matches = App::new("Hellman")
        .version("1.0")
        .about("Does awesome things")
        .arg(Arg::new("pem_path")
            .about("Sets the pem file to use")
            .required(true)
            .index(1))
*/
//todo: send bsc
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let matches = App::new("bot")
        .version("1.0")
        .about("Does awesome things")
        .arg(Arg::new("pri_key")
            .required(true)
            .index(1))
        .arg(Arg::new("price")
            .required(true)
            .index(2))
        .arg(Arg::new("amount")
            .required(true)
            .index(3))
        .arg(Arg::new("side")
            .required(true)
            .index(4))
        .get_matches();
    let pri_key : &str = matches.value_of("pri_key").unwrap();
    let price_str : &str = matches.value_of("price").unwrap();
    let amount_str : &str = matches.value_of("amount").unwrap();




    let base_price = 1001.0f64;
    let base_amount = 1.0f64;
    //get_dex_name().await;

    //test1,0x613548d151E096131ece320542d19893C4B8c901
    let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
    //test2,0x37BA121cdE7a0e24e483364185E80ceF655346DD
    let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";
    //test3,0xca9B361934fc7A7b07814D34423d665268111726
    let pri_key = "b0a09e85dad814ccc7231982401cca5accc3a46bc68349b403a7a129517cc266";


    //tj
    //let pri_key = "1f3bc7d273c179f0b73745d0599a15ece081837a9aa4ccb6351842fcad19fb95";
    //local
   let pri_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let pri_key : &str = matches.value_of("pri_key").unwrap();

    /***
deployTokenA:   0xc739cD8920C65d372a0561507930aB6993c33E30
deployTokenB:   0x1982C0fC743078a7484bd82AC7A17BDab344308e
deployStorage:   0xAfC8a33002B274F43FC56D28D515406966354388
deployTokenProxy:   0x913e9d1a60bEb3312472A53CAe1fe64bC4df60e2
deployVault:   0x003fDe97E3a0932B2Bc709e952C6C9D73E0E9aE4
deployChemiMain:   0x0f48DDFe03827cd5Efb23122B44955c222eCd720

deployTokenA:   0xb8a1255FB1d23EF1BEedf3c7024CfB178e7bA7B4
deployTokenB:   0xCdE5A755aCdc7db470F206Ea98F802E42903C4f2



 * deployTokenA:   0x5FbDB2315678afecb367f032d93F642f64180aa3
 * deployTokenB:   0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512

 --deployTokenA:   0x38F52517e6642fB1933E7A6A3a34fEa35372eD32
--deployTokenB:   0x719d36AB3752aa2d0311637B79B480C00A8f83fC
deployTokenA:   0x78F8D152Dc041E6Aa027342A12D19EF9ecf5038a
deployTokenB:   0xCB40288aF19767c0652013D3072e0Dd983d0cFFE
    */

    let chemix_main_addr = env::CONF.chemix_main.to_owned().unwrap();
    let base_token = "0x38F52517e6642fB1933E7A6A3a34fEa35372eD32"; //AAA 18
    let quote_token = "0x719d36AB3752aa2d0311637B79B480C00A8f83fC"; //BBB 15

   //loop {
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

        let side : &str = matches.value_of("side").unwrap();
        let price= price_str.to_string().parse::<f64>().unwrap();
        let amount= amount_str.to_string().parse::<f64>().unwrap();

    println!(
            "[newOrder]: side {} price {},amount {}",
            side, price, amount
        );
        let client = ChemixContractClient::new(pri_key, chemix_main_addr.to_str().unwrap());
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
        /***
            loop {
                match client.cancel_order(base_token,quote_token,49).await {
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

         */







        tokio::time::sleep(time::Duration::from_millis(1000)).await;
   //}
    Ok(())
}
