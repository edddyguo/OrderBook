use anyhow::Result;

use chrono::Local;

use common::env::CONF as ENV_CONF;

use common::utils::algorithm::u8_arr_to_str;

use ethers::prelude::*;
use ethers::types::Address;
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

use std::str::FromStr;
use ethers::abi::Detokenize;
use ethers::prelude::builders::ContractCall;

use crate::chemix::ChemixContractClient;
use crate::{contract_call_send, gen_contract_client, sign_tx, TypedTransaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThawBalances {
    pub token: Address,
    pub from: Address,
    pub amount: U256,
    pub decimal: u32,
}

#[derive(Clone, Debug)]
pub struct SettleValues3 {
    pub user: Address,
    pub token: Address,
    pub isPositive: bool,
    pub incomeTokenAmount: U256,
}

#[derive(Clone)]
pub struct Vault {}

abigen!(
    ChemixVault,
    "../contract/Vault.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

lazy_static! {
    static ref VAULT_ADDR: Address = {
        let vault = ENV_CONF.chemix_vault.to_owned().unwrap();
        Address::from_str(vault.to_str().unwrap()).unwrap()
    };
}

impl ChemixContractClient<Vault> {
    pub fn new(prikey: &str) -> ChemixContractClient<Vault> {
        ChemixContractClient {
            client: gen_contract_client(prikey),
            contract_addr: VAULT_ADDR.clone(),
            phantom: PhantomData,
        }
    }

    pub async fn thaw_balances(
        &self,
        users: Vec<ThawBalances>,
        cancel_id: [u8; 32],
    ) -> Bytes {
        let contract = ChemixVault::new(self.contract_addr, self.client.clone());

        let users = users
            .iter()
            .map(|x| ThawInfos {
                token: x.token,
                addr: x.from,
                thaw_amount: x.amount,
            })
            .collect::<Vec<ThawInfos>>();

        let mut call = contract
            .thaw_balance(cancel_id, users)
            .legacy();
        sign_tx(&mut call.tx).await
    }

    pub async fn settlement_trades2(
        &self,
        last_index: u32,
        last_hash: [u8; 32],
        trades: Vec<SettleValues3>,
    ) -> Bytes {
        let contract = ChemixVault::new(self.contract_addr, self.client.clone());
        let trades2 = trades
            .iter()
            .map(|x| SettleValues {
                user: x.user,
                token: x.token,
                is_positive: x.isPositive,
                income_token_amount: x.incomeTokenAmount,
            })
            .collect::<Vec<SettleValues>>();

        let mut call = contract
            .settlement(U256::from(last_index), last_hash, trades2)
            .legacy();
        sign_tx(&mut call.tx).await
    }

    pub async fn filter_settlement_event(&mut self, block_hash: H256) -> Result<Vec<String>> {
        let contract = ChemixVault::new(self.contract_addr, self.client.clone());
        let new_orders: Vec<SettlementFilter> = contract
            .settlement_filter()
            .at_block_hash(block_hash)
            .query()
            .await
            .unwrap();

        let settlement_flag = new_orders
            .iter()
            .map(|x| u8_arr_to_str(x.hash_data))
            .collect::<Vec<String>>();
        Ok(settlement_flag)
    }

    //thaws
    pub async fn filter_thaws_event(&mut self, block_hash: H256) -> Result<Vec<String>> {
        let contract = ChemixVault::new(self.contract_addr, self.client.clone());
        let new_orders: Vec<ThawBalanceFilter> = contract
            .thaw_balance_filter()
            .at_block_hash(block_hash)
            .query()
            .await
            .unwrap();

        let thaws_flag = new_orders
            .iter()
            .map(|x| u8_arr_to_str(x.flag))
            .collect::<Vec<String>>();
        Ok(thaws_flag)
    }

    pub async fn vault_balance_of(
        &mut self,
        token: String,
        from: String,
    ) -> Result<(U256, U256)> {
        let contract = ChemixVault::new(self.contract_addr, self.client.clone());
        let token = Address::from_str(token.as_str()).unwrap();
        let from = Address::from_str(from.as_str()).unwrap();
        let value = contract.balance_of(token, from).call().await?;
        info!("vault_balance_of result  {:?}", value);
        Ok(value)
    }

    pub async fn vault_total_withdraw_volume(&self, token: String) -> Result<U256> {
        let contract = ChemixVault::new(self.contract_addr, self.client.clone());
        let token = Address::from_str(token.as_str()).unwrap();
        let value = contract.total_withdraw(token).call().await?;
        info!("vault_balance_of result  {:?}", value);
        Ok(value)
    }
}
