// use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
use ethers::prelude::*;
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_providers::{Middleware, Provider, StreamExt, Ws};

//let ws = Ws::connect("wss://localhost:8545").await?;

// Use the `tokio::main` macro for using async on the main function
pub fn sign() -> Result<()> {
    println!("in sign");
    Ok(())
}

pub async fn listen_block() -> anyhow::Result<()> {
    //let ganache = Ganache::new().block_time(1u64).spawn();
    //let ws = Ws::connect(ganache.ws_endpoint()).await?;
    let ws = Ws::connect("wss://rinkeby.infura.io/ws/v3/c60b0bb42f8a4c6481ecd229eddaca27")
        .await
        .unwrap();
    let provider = Provider::new(ws).interval(Duration::from_millis(2000));
    let mut stream = provider.watch_blocks().await?.take(5);
    while let Some(block) = stream.next().await {
        println!("in sign2");
        dbg!(block);
    }

    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
