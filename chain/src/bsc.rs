use anyhow::Result;
use ethers::prelude::*;

//use ethers::providers::Ws;
use crate::Node;
use ethers_providers::{Middleware, Ws};

pub async fn get_block<T: Into<BlockId> + Send + Sync>(
    height_or_hash: T,
) -> Result<Option<Block<H256>>> {
    match crate::PROVIDER.provide.get_block(height_or_hash).await? {
        None => Ok(None),
        Some(block) => Ok(Some(block)),
    }
}

pub async fn gen_watcher() -> Node<Ws> {
    Node::<Ws>::new(crate::WATCHER.clone().as_str()).await
}

pub async fn get_current_block() -> u32 {
    let mut height;
    loop {
        match  crate::PROVIDER.provide.get_block_number().await {
            Ok(num) => {
                height = num.as_u32();
                break;
            }
            Err(error) => {
                warn!("get_current_block failed {:?}",error);
                continue;
            }
        }
    }
    height
}
