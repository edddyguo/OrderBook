use crate::k256::ecdsa::SigningKey;
use anyhow::Result;
use chemix_models::order::BookOrder;
use chrono::Local;
use common::env;
use common::env::CONF as ENV_CONF;
use common::types::order::Side;
use common::types::*;
use common::utils::algorithm::{sha256, u8_arr_to_str};
use common::utils::math::MathOperation;
use ethers::prelude::*;
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::Mul;
use std::str::FromStr;
use std::sync::Arc;
use crate::{ContractClient, gen_contract_client};
use crate::chemix::ChemixContractClient;

#[derive(Clone)]
pub struct Main {}

abigen!(
    ChemixMain,
    "../contract/ChemixMain.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

lazy_static! {
    static ref MAIN_ADDR : Address = {
       let main = ENV_CONF.chemix_main.to_owned().unwrap();
        Address::from_str(main.to_str().unwrap()).unwrap()
    };
}


impl ChemixContractClient<Main> {
    pub fn new(prikey: &str) -> ChemixContractClient<Main> {
        ChemixContractClient {
            client: gen_contract_client(prikey),
            contract_addr: MAIN_ADDR.clone(),
            phantom: PhantomData,
        }
    }

    pub async fn new_order(
        &self,
        side: Side,
        baseToken: &str,
        quoteToken: &str,
        price: f64,
        amount: f64,
    ) -> Result<()> {
        let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let tokenADecimal = U256::from(10u128).pow(U256::from(10u32)); //18 -8
        let tokenBDecimal = U256::from(10u128).pow(U256::from(7u32)); //15 -8

        let quoteToken = Address::from_str(quoteToken).unwrap();
        let baseToken = Address::from_str(baseToken).unwrap();

        let amount = U256::from(amount.to_nano()).mul(tokenADecimal);
        let price = U256::from(price.to_nano()).mul(tokenBDecimal);
        match side {
            Side::Buy => {
                info!(
                    "new_limit_buy_order,quoteToken={},baseToken={},price={},amount={}",
                    quoteToken, baseToken, price, amount
                );
                let result = contract
                    .new_limit_buy_order(
                        baseToken,
                        quoteToken,
                        price,
                        amount,
                        U256::from(18u32),
                    )
                    .legacy()
                    .send()
                    .await?
                    .await?;
                info!("new buy order result  {:?}", result.unwrap().block_number);
            }
            Side::Sell => {
                info!(
                    "new_limit_sell_order,quoteToken={},baseToken={},price={},amount={}",
                    quoteToken, baseToken, price, amount
                );
                let result = contract
                    .new_limit_sell_order(
                        baseToken,
                        quoteToken,
                        price,
                        amount,
                        U256::from(18u32),
                    )
                    .legacy()
                    .send()
                    .await?
                    .await?;
                info!("new sell order result  {:?}", result.unwrap().block_number);
            }
        }
        Ok(())
    }

    pub async fn cancel_order(
        &self,
        baseToken: &str,
        quoteToken: &str,
        order_index: u32,
    ) -> Result<()> {
        let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let quoteToken = Address::from_str(quoteToken).unwrap();
        let baseToken = Address::from_str(baseToken).unwrap();

        info!(
            "cancel_order market: {}-{} order_index: {}",
            baseToken, quoteToken, order_index
        );
        let result = contract
            .new_cancel_order(baseToken, quoteToken, U256::from(order_index))
            .legacy()
            .send()
            .await?
            .await?;
        info!("new sell order result  {:?}", result);
        Ok(())
    }
}



