use std::convert::TryFrom;
use anyhow::Result;
use ethers::prelude::*;
use std::time::Duration;
//use ethers::providers::Ws;
use ethers_providers::{Middleware, Provider, StreamExt, Ws, Http};

#[derive(Clone, Debug)]
pub struct Node<P> {
    provide: Provider<P>,
}

//"ws://58.33.12.252:7548/"
//"http://58.33.12.252:8548";
impl Node<Http> {
    pub fn new(host: &str) -> Node<Http> {
        Node {
            provide: Provider::<Http>::try_from(host).unwrap()
        }
    }

    pub async fn get_block<T: Into<BlockId> + Send + Sync>(&self, height_or_hash: T) -> Result<Option<U64>> {
        match self.provide.get_block(height_or_hash).await?{
            None => {
                Ok(None)
            },
            Some(block) => {
                Ok(block.number)
            }
        }
    }
}

impl Node<Ws> {
    pub async fn new(host: &str) -> Node<Ws> {
        let ws = Ws::connect(host).await.unwrap();
        Node {
            provide: Provider::new(ws).interval(Duration::from_millis(2000))
        }
    }

    pub async fn gen_watcher(&self) -> Result<FilterWatcher<'_,Ws, H256>> {
        Ok(self.provide.watch_blocks().await?)
    }
}

/***
impl Node<P>{
    fn get_block(&self, height: u64) -> Result<u64> {
        let Http2(provide) = self::Http2;
        Ok(provide.get_block(U64::from(height)).await??.number?.as_u64())
    }

    fn gen_watcher<P>(&self, height: u64) -> Result<u64> {
        let Ws2(watcher) = self::Http2;
        Ok(provide.get_block(U64::from(height)).await??.number?.as_u64())
    }
}

 */