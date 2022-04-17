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
use crate::types::transaction::eip2718::TypedTransaction;
use ethers::abi::Detokenize;
use ethers::contract::builders::ContractCall;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::core::utils::keccak256;
use std::sync::Arc;
use std::time;

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
    static ref WATCHER: String = {
        let url = ENV_CONF.chain_ws.to_owned().unwrap();
        url.to_str().unwrap().to_owned()
    };
    static ref PROVIDER: Node<Http> = {
        let url = ENV_CONF.chain_rpc.to_owned().unwrap();
        Node::<Http>::new(url.to_str().unwrap())
    };
    static ref CHAIN_ID: u64 = {
        let chain_id = ENV_CONF.chain_id.to_owned().unwrap();
        chain_id.into_string().unwrap().parse::<u64>().unwrap()
    };
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
        .with_chain_id(*crate::CHAIN_ID);
    Arc::new(SignerMiddleware::new(PROVIDER.provide.clone(), wallet))
}

//机器人暂时不需要校验txid
async fn contract_call_send<D: Detokenize, M: Middleware>(
    call: ContractCall<M, D>,
) -> Result<TransactionReceipt, ProviderError> {
    loop {
        let _hash = call.tx.sighash(U64::from(*crate::CHAIN_ID));
        let mut times = 10;
        while times != 0 {
            let signature1 = crate::CONTRACT_CLIENT
                .sign_transaction(&call.tx, Address::default())
                .await
                .unwrap();
            info!("signature1 = {:?}", signature1);
            times -= 1;
        }

        let signature = crate::CONTRACT_CLIENT
            .sign_transaction(&call.tx, Address::default())
            .await
            .unwrap();
        let txid2 = call.tx.rlp_signed(U64::from(*crate::CHAIN_ID), &signature);
        let txid3: H256 = keccak256(txid2).into();
        info!("local txid {:?}", txid3);
        match call.send().await.unwrap().await {
            Ok(data) => {
                info!("remote txid {:?}", data.as_ref().unwrap().transaction_hash);
                return Ok(data.unwrap());
            }
            Err(error) => {
                if error.to_string().contains("underpriced") {
                    warn!("gas too low and try again");
                    tokio::time::sleep(time::Duration::from_millis(1000)).await;
                } else {
                    error!("{}", error.to_string());
                    return Err(error);
                }
            }
        }
    }
}

//todo，暂时节点故障直接unwarp
async fn sign_tx(transaction: &mut TypedTransaction) -> Bytes {
    let _res = crate::CONTRACT_CLIENT
        .fill_transaction(transaction, None)
        .await
        .unwrap();
    info!("[text_txid]:: transaction1 {:?}", transaction);
    let signature = crate::CONTRACT_CLIENT
        .sign_transaction(transaction, Address::default())
        .await
        .unwrap();
    transaction.rlp_signed(*crate::CHAIN_ID, &signature)
}

pub fn gen_txid(data: &Bytes) -> String {
    let hash: H256 = keccak256(data).into();
    let txid = format!("{:?}", hash);
    txid
}

//todo: 异常处理
pub async fn send_raw_transaction(tx3: Bytes) -> TransactionReceipt {
    let receipt = crate::CONTRACT_CLIENT
        .send_raw_transaction(tx3)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap();
    info!("remote txid {:?}", receipt.transaction_hash);
    receipt
}
#[cfg(test)]
mod tests {
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::core::types::TransactionRequest;
    use ethers::signers::LocalWallet;
    use ethers::utils::keccak256;
    use std::convert::TryFrom;

    use ethers::prelude::*;

    //use ethers::providers::Ws;

    use crate::types::transaction::eip2718::TypedTransaction;
    use ethers_providers::{Http, Middleware, Provider};

    #[tokio::test]
    async fn signs_tx() {
        // retrieved test vector from:
        // https://web3js.readthedocs.io/en/v1.2.0/web3-eth-accounts.html#eth-accounts-signtransaction
        let tx = TransactionRequest {
            from: Some(
                "91352ab925F5ee09937F2E7753b243C648a975C4"
                    .parse::<Address>()
                    .unwrap(),
            ),
            to: Some(
                "F0109fC8DF283027b6285cc889F5aA624EaC1F55"
                    .parse::<Address>()
                    .unwrap()
                    .into(),
            ),
            value: Some(U256::from(1u64)),
            gas: Some(U256::from(2_000_000u64)),
            nonce: Some(U256::from(3u64)),
            gas_price: Some(U256::from(21_000_000_000u128)),
            data: None,
        };
        let chain_id = 15u64;

        //let provider = Provider::try_from("http://localhost:8545").unwrap();
        let provider = Provider::<Http>::try_from("http://192.168.1.158:8548").unwrap();
        let key = "29958aa55f0539e8108fd6cc605281a3367f7d562669ba9e31b2c1772cd7bb57"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id);

        let client: SignerMiddleware<Provider<Http>, Wallet<SigningKey>> =
            SignerMiddleware::new(provider, key);
        let signature = client
            .sign_transaction(&TypedTransaction::Legacy(tx.clone()), Address::default())
            .await
            .unwrap();
        let tx3 = tx.rlp_signed(&signature);
        let txid: H256 = keccak256(&tx3).into();
        println!("local txid{:?}", txid);
        let receipt = client
            .send_raw_transaction(tx3)
            .await
            .unwrap()
            .await
            .unwrap()
            .unwrap();
        println!("remote txid {:?}", receipt.transaction_hash);
    }
}
