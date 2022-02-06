use anyhow::Result;
use ethers::{prelude::*, utils::Ganache};
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_providers::{Ws, Provider, Middleware, StreamExt, Http};
use std::convert::TryFrom;
use std::str::FromStr;
use tokio::time;
use std::ops::Add;
use std::sync::{Arc, mpsc, RwLock};
use ethers_contract_abigen::{parse_address, Address};
use tokio::runtime::Runtime;
use rsmq_async::{Rsmq, RsmqConnection};


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
    eprintln!("balance {}", balance_before);
    Ok(())
}

async fn listen_blocks() -> anyhow::Result<()> {
    //let host = "https://bsc-dataseed4.ninicoin.io";
    //testnet
    let host = "https://data-seed-prebsc-2-s3.binance.org:8545";

    let provider_http = Provider::<Http>::try_from(host).unwrap();

    let mut rsmq = Rsmq::new(Default::default())
        .await
        .expect("connection failed");

    //todo: wss://bsc-ws-node.nariox.org:443
    /***
    let ws = Ws::connect("wss://bsc-ws-node.nariox.org:443/").await.unwrap();
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?;
    while let Some(block) = stream.next().await {
        let block_content = provider_http.get_block(block).await.unwrap();
        println!("block content {:?}",block_content);
    }
     */
    let wallet = "1b03a06c4a89d570a8f1d39e9ff0be8891f7657898675f11585aa7ec94fe2d12"
        .parse::<LocalWallet>()
        .unwrap();
    let address = wallet.address();
    println!("wallet address {:?}", address);
    let mut height = provider_http.get_block_number().await.unwrap();
    let mut height: U64 = U64::from(16477780u64);
    let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
    let client = Arc::new(client);

    let (event_sender, event_receiver) = mpsc::sync_channel(0);
    rayon::scope(|s| {
        //send event in new block
        s.spawn(move |_| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async move {
                loop {
                    dbg!(height);
                    let block_content = provider_http.get_block(height).await.unwrap();
                    if block_content.is_none() {
                        tokio::time::sleep(time::Duration::from_secs(2)).await;
                        println!("block not found,and wait a moment");
                    } else {
                        let addr = parse_address("0xE41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C").unwrap();
                        let contract = SimpleContract::new(addr, client.clone());
                        let logs: Vec<NewOrderFilter> = contract.new_order_filter().from_block(height.as_u64()).query().await.unwrap();
                        event_sender.send(logs).expect("failed to send orders");
                        //block content logs [NewOrderFilter { user: 0xfaa56b120b8de4597cf20eff21045a9883e82aad, base_token: "BTC", quote_token: "USDT", amount: 3, price: 4 }]
                        //println!("New order Event {:?},base token {:?}",logs[0].user,logs[0].base_token);
                        //height = height.add(1);
                    }
                }
            });
        });
        s.spawn(move |_| {
            let mut arc_rsmq = Arc::new(RwLock::new(rsmq));
            loop {
                let mut arc_rsmq = arc_rsmq.clone();
                let orders: Vec<NewOrderFilter> = event_receiver.recv().expect("failed to recv columns");
                println!("[listen_blocks: receive] New order Event {:?},base token {:?}", orders[0].user, orders[0].base_token);
                let json_str = serde_json::to_string(&orders).unwrap();
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    arc_rsmq.write().unwrap().send_message("myqueue", json_str, None)
                        .await
                        .expect("failed to send message");
                });

            }
        });
    });

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    /***
    rsmq.create_queue("myqueue", None, None, None)
        .await
        .expect("failed to create queue");
    ***/

    listen_blocks().await;
    Ok(())
}