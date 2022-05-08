use std::str::FromStr;
use anyhow::Result;
use ethers::prelude::*;

//use ethers::providers::Ws;
use crate::Node;
use ethers_providers::{Middleware, Ws};

///get block by block height or block hash
pub async fn get_block<T: Into<BlockId> + Send + Sync>(
    height_or_hash: T,
) -> Result<Option<Block<H256>>> {
    match crate::PROVIDER.provide.get_block(height_or_hash).await? {
        None => Ok(None),
        Some(block) => Ok(Some(block)),
    }
}

///generate chain ws watcher client
pub async fn gen_watcher() -> Node<Ws> {
    Node::<Ws>::new(crate::WATCHER.clone().as_str()).await
}

///get last block height
pub async fn get_current_block() -> u32 {
    crate::PROVIDER
        .provide
        .get_block_number()
        .await
        .unwrap()
        .as_u32()
}


///get block height by hash
pub async fn transaction_at(hash: &str) -> Option<u64> {
    let tx_hash = TxHash::from_str(hash).unwrap();
    //检查event
    crate::PROVIDER
        .provide
        .get_transaction(tx_hash).await.unwrap()
        .and_then(
            |x| x.block_number.map(|x| x.as_u64())
        )

}
