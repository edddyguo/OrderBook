use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_providers::{Ws, Provider, Middleware, StreamExt, Http};
use std::convert::TryFrom;
use std::str::FromStr;


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
    //wss://bsc-ws-node.nariox.org:443
    let ws = Ws::connect("wss://bsc-ws-node.nariox.org:443/").await.unwrap();
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?;
    while let Some(block) = stream.next().await {
        println!("in sign2");
        dbg!(block);
        let block_content = provider_http.get_block(block).await.unwrap();
        println!("block content {:?}",block_content);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    listen_blocks().await;
    Ok(())
}