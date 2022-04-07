use anyhow::Result;

use common::env::CONF as ENV_CONF;
use common::teen_power;
use common::types::order::Side;

use common::utils::math::MathOperation;
use ethers::prelude::*;
use ethers::types::Address;

use std::marker::PhantomData;
use std::ops::{Mul};
use std::str::FromStr;

use crate::chemix::ChemixContractClient;
use crate::{contract_call_send, gen_contract_client};

#[derive(Clone)]
pub struct Main {}

abigen!(
    ChemixMain,
    "../contract/ChemixMain.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

lazy_static! {
    static ref MAIN_ADDR: Address = {
        let main = ENV_CONF.chemix_main.to_owned().unwrap();
        Address::from_str(main.to_str().unwrap()).unwrap()
    };
}

impl ChemixContractClient<Main> {
    pub fn new(prikey: &str) -> ChemixContractClient<Main> {
        ChemixContractClient {
            client: gen_contract_client(prikey),
            contract_addr: *MAIN_ADDR,
            phantom: PhantomData,
        }
    }

    pub async fn new_order(
        &self,
        side: Side,
        base_token: &str,
        quote_token: &str,
        price: f64,
        amount: f64,
    ) -> Result<()> {
        let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let base_decimal = teen_power!(10u32); //18 -8
        let quote_decimal = teen_power!(7u32); //15 -8

        let quote_token = Address::from_str(quote_token).unwrap();
        let base_token = Address::from_str(base_token).unwrap();

        let amount = U256::from(amount.to_nano()).mul(base_decimal);
        let price = U256::from(price.to_nano()).mul(quote_decimal);
        match side {
            Side::Buy => {
                info!(
                    "new_limit_buy_order,quoteToken={},baseToken={},price={},amount={}",
                    quote_token, base_token, price, amount
                );
                let result = contract
                    .new_limit_buy_order(
                        base_token,
                        quote_token,
                        price,
                        amount,
                        U256::from(18u32),
                    )
                    .legacy()
                    .send()
                    .await?
                    .await?;
                info!(
                    "new buy order result: block={:?},txid={:?}",
                    result.as_ref().unwrap().block_number,
                    result.as_ref().unwrap().transaction_hash
                );
            }
            Side::Sell => {
                info!(
                    "new_limit_sell_order,quoteToken={},baseToken={},price={},amount={}",
                    quote_token, base_token, price, amount
                );
                let result = contract
                    .new_limit_sell_order(
                        base_token,
                        quote_token,
                        price,
                        amount,
                        U256::from(18u32),
                    )
                    .legacy()
                    .send()
                    .await?
                    .await?;
                info!(
                    "new sell order result: block={:?},txid={:?}",
                    result.as_ref().unwrap().block_number,
                    result.as_ref().unwrap().transaction_hash
                );
            }
        }
        Ok(())
    }

    pub async fn cancel_order(
        &self,
        base_token: &str,
        quote_token: &str,
        order_index: u32,
    ) -> std::result::Result<TransactionReceipt, ProviderError> {
        let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let quote_token = Address::from_str(quote_token).unwrap();
        let base_token = Address::from_str(base_token).unwrap();

        info!(
            "cancel_order market: {}-{} order_index: {}",
            base_token, quote_token, order_index
        );
        let call = contract
            .new_cancel_order(base_token, quote_token, U256::from(order_index))
            .legacy();
        contract_call_send(call).await
    }
}
