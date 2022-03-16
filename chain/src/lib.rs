pub mod bsc;
pub mod chemix;

use std::convert::TryFrom;
// use the anyhow crate for easy idiomatic error handling

use ethers::prelude::*;
use std::time::Duration;
//use ethers::providers::Ws;
use common::env::CONF as ENV_CONF;
use ethers_providers::{Http, Middleware, StreamExt, Ws};

type ContractClient = Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;
use ethers::core::k256::ecdsa::SigningKey;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Node<P> {
    pub provide: Provider<P>,
}

//"ws://58.33.12.252:7548/"
//"http://58.33.12.252:8548";
impl Node<Http> {
    pub fn new(host: &str) -> Node<Http> {
        Node {
            provide: Provider::<Http>::try_from(host).unwrap(),
        }
    }
}

impl Node<Ws> {
    pub async fn new(host: &str) -> Node<Ws> {
        let ws = Ws::connect(host).await.unwrap();
        Node {
            provide: Provider::new(ws).interval(Duration::from_millis(2000)),
        }
    }
}

lazy_static! {
    static ref WATCHER : String = {
            let url = ENV_CONF.chain_ws.to_owned().unwrap();
            url.to_str().unwrap().to_owned()
    };

    static ref PROVIDER : Node<Http> =  {
            let url = ENV_CONF.chain_rpc.to_owned().unwrap();
            Node::<Http>::new(url.to_str().unwrap())
    };

    static ref CHAIN_ID : u64 = {
            let chain_id = ENV_CONF.chain_id.to_owned().unwrap();
            chain_id.into_string().unwrap().parse::<u64>().unwrap()
    };

    //fixme: 放到上层注入
    static ref CONTRACT_CLIENT: ContractClient = {
        let chain_id = ENV_CONF.chain_id.to_owned().unwrap();
        let chain_id = chain_id.into_string().unwrap().parse::<u64>().unwrap();

        let prikey = ENV_CONF.chemix_relayer_prikey.to_owned().unwrap();
        let prikey_str = prikey.to_str().unwrap().to_owned();

        let wallet = prikey_str
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);
        Arc::new(SignerMiddleware::new(PROVIDER.provide.clone(), wallet))
    };
}

pub async fn listen_block() -> anyhow::Result<()> {
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

fn gen_contract_client(prikey_str: &str) -> ContractClient {
    let wallet = prikey_str
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(crate::CHAIN_ID.clone());
    Arc::new(SignerMiddleware::new(PROVIDER.provide.clone(), wallet))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
