use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_providers::{Ws, Provider, Middleware, StreamExt, Http};
use std::convert::TryFrom;
use std::str::FromStr;
use tokio::time;
use std::ops::Add;
use std::sync::Arc;
use ethers_contract_abigen::{parse_address, Address};

/***

// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;
    event NewOrder(address user, string baseToken, string quoteToken ,uint amount, uint price);

}
*/

#[derive(Debug, PartialEq, EthEvent)]
pub struct NewOrderEvent {
    user: Address,
    baseToken: String,
    quoteToken: String,
    amount: u64,
    price: u64,
}



abigen!(
    SimpleContract,
    "../contract/chemix_trade_abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub fn sign() -> Result<()> {
    println!("in sign");
    Ok(())
}


async fn get_balance() -> Result<()> {
    let host = "https://mainnet.infura.io/v3/8b4e814a07474456828cc110195adca2";
    let provider_http = Provider::<Http>::try_from(host).unwrap();
    let addr = "90a97d253608B2090326097a44eA289d172c30Ec".parse().unwrap();
    let union = NameOrAddress::Address(addr);
    let balance_before = provider_http.get_balance(union, None).await?;
    eprintln!("balance {}",balance_before);
    Ok(())
}

async fn listen_blocks() -> anyhow::Result<()> {
    //let host = "https://bsc-dataseed4.ninicoin.io";
    let host = "https://data-seed-prebsc-2-s3.binance.org:8545";

    let provider_http = Provider::<Http>::try_from(host).unwrap();
    //todo: wss://bsc-ws-node.nariox.org:443
    /***
    let ws = Ws::connect("wss://bsc-ws-node.nariox.org:443/").await.unwrap();
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?;
    while let Some(block) = stream.next().await {
        println!("in sign2");
        dbg!(block);
        let block_content = provider_http.get_block(block).await.unwrap();
        println!("block content {:?}",block_content);
    }
     */
    let wallet = "1b03a06c4a89d570a8f1d39e9ff0be8891f7657898675f11585aa7ec94fe2d12"
        .parse::<LocalWallet>()
        .unwrap();
    let address = wallet.address();
    println!("wallet address {:?}",address);

    let mut height = provider_http.get_block_number().await.unwrap();
    let mut height : U64 = U64::from(16477780u64);
    loop {
        dbg!(height);
        let block_content = provider_http.get_block(height).await.unwrap();
        if block_content.is_none() {
            tokio::time::sleep(time::Duration::from_secs(2)).await;
            println!("block not found,and wait a moment");
        }else {

            let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
            let client = Arc::new(client);

            let addr = parse_address("0xE41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C").unwrap();
            let contract = SimpleContract::new(addr, client.clone());
            let logs = contract.new_order_filter().from_block(height.as_u64()).query().await?;
            //block content logs [NewOrderFilter { user: 0xfaa56b120b8de4597cf20eff21045a9883e82aad, base_token: "BTC", quote_token: "USDT", amount: 3, price: 4 }]

            //println!("block content logs {:?}",logs);
            //let event = <NewOrderEvent as EthLogDecode>::decode_log(&logs).unwrap();
            println!("New order Event {:?},base token {:?}",logs[0].user,logs[0].base_token);


            //height = height.add(1);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    listen_blocks().await;
    Ok(())
}

/***
 #[derive(Debug, PartialEq, EthEvent)]
    pub struct LiquidateBorrow {
        liquidator: Address,
        borrower: Address,
        repay_amount: U256,
        c_token_collateral: Address,
        seize_tokens: U256,
    }
    // https://etherscan.io/tx/0xb7ba825294f757f8b8b6303b2aef542bcaebc9cc0217ddfaf822200a00594ed9#eventlog index 141
    let log = RawLog {
        topics: vec!["298637f684da70674f26509b10f07ec2fbc77a335ab1e7d6215a4b2484d8bb52"
            .parse()
            .unwrap()],
        data: vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 188, 205, 0, 29, 173, 151, 238, 5, 127, 91, 31,
            197, 154, 221, 40, 175, 143, 32, 26, 201, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 133, 129,
            195, 136, 163, 5, 24, 136, 69, 34, 251, 23, 122, 146, 252, 33, 147, 81, 8, 20, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 18, 195, 162, 210,
            38, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 77, 220, 45, 25, 57, 72, 146, 109, 2,
            249, 177, 254, 158, 29, 170, 7, 24, 39, 14, 213, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 80, 30, 88,
        ],
    };
    let event = <LiquidateBorrow as EthLogDecode>::decode_log(&log).unwrap();
    assert_eq!(event.seize_tokens, 5250648u64.into());
    assert_eq!(event.repay_amount, 653800000000000000u64.into());

*/