use anyhow::Result;
use ethers::prelude::*;
use std::convert::TryFrom;
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_providers::{Http, Middleware, Provider, Ws};
use crate::Node;

pub async fn get_block<T: Into<BlockId> + Send + Sync>(
    height_or_hash: T,
) -> Result<Option<U64>> {
    match crate::PROVIDER.provide.get_block(height_or_hash).await? {
        None => Ok(None),
        Some(block) => Ok(block.number),
    }
}

pub async fn gen_watcher() -> Node<Ws> {
    Node::<Ws>::new(crate::WATCHER.clone().as_str()).await
}



pub async fn get_current_block() -> u32 {
    crate::PROVIDER.provide.get_block_number().await.unwrap().as_u32()
}

