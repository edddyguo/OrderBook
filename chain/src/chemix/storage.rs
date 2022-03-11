use anyhow::Result;

use chrono::Local;

use common::env::CONF as ENV_CONF;

use common::types::*;
use common::utils::algorithm::{sha256, u8_arr_to_str};

use ethers::prelude::*;
use ethers::types::Address;

use std::marker::PhantomData;

use std::str::FromStr;

use crate::chemix::ChemixContractClient;
use crate::gen_contract_client;
use common::types::order::Side as OrderSide;

#[derive(Clone, Debug)]
pub struct CancelOrderState2 {
    pub base_token: Address,
    pub quote_token: Address,
    pub order_user: Address,
    pub cancel_index: U256,
    pub order_index: U256,
    pub hash_data: [u8; 32],
}

//  event NewOrderCreated(address indexed quoteToken, address indexed baseToken,
//                             bytes32 indexed hashData, address orderUser, bool orderType, uint256 orderIndex,
//                             uint256 limitPrice, uint256 orderAmount);
#[derive(Clone, Debug)]
pub struct ChainNewOrder {
    pub id: String,
    pub transaction_hash: String,
    pub account: String,
    pub index: u32,
    pub num_power: u32,
    pub hash_data: String,
    pub side: OrderSide,
    pub price: U256,
    pub amount: U256,
}

abigen!(
    ChemixStorage,
    "../contract/ChemixStorage.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

lazy_static! {
    static ref STORAGE_ADDR: Address = {
        let storage = ENV_CONF.chemix_storage.to_owned().unwrap();
        Address::from_str(storage.to_str().unwrap()).unwrap()
    };
}

#[derive(Clone)]
pub struct Storage {}

impl ChemixContractClient<Storage> {
    pub fn new(prikey: &str) -> ChemixContractClient<Storage> {
        ChemixContractClient {
            client: gen_contract_client(prikey),
            contract_addr: STORAGE_ADDR.clone(),
            phantom: PhantomData,
        }
    }

    pub async fn filter_new_cancel_order_created_event(
        &mut self,
        height: u32,
        base_token: String,
        quote_token: String,
    ) -> Result<Vec<CancelOrderState2>> {
        let base_token = Address::from_str(base_token.as_str()).unwrap();
        let quote_token = Address::from_str(quote_token.as_str()).unwrap();

        let contract = ChemixStorage::new(self.contract_addr, self.client.clone());
        let canceled_orders: Vec<NewCancelOrderCreatedFilter> = contract
            .new_cancel_order_created_filter()
            .from_block(U64::from(height))
            .query()
            .await
            .unwrap();
        let new_orders2 = canceled_orders
            .iter()
            .filter(|x| x.base_token == base_token && x.quote_token == quote_token)
            .map(|x| CancelOrderState2 {
                base_token: x.base_token,
                quote_token: x.quote_token,
                order_user: x.cancel_user,
                cancel_index: x.m_cancel_index,
                order_index: x.order_index,
                hash_data: x.hash_data,
            })
            .collect::<Vec<CancelOrderState2>>();
        Ok(new_orders2)
    }

    pub async fn filter_new_order_event(
        &mut self,
        height: u32,
        base_token: String,
        quote_token: String,
    ) -> Result<Vec<ChainNewOrder>> {
        let base_token = Address::from_str(base_token.as_str()).unwrap();
        let quote_token = Address::from_str(quote_token.as_str()).unwrap();
        let contract = ChemixStorage::new(self.contract_addr, self.client.clone());
        let new_orders: Vec<(NewOrderCreatedFilter,LogMeta)> = contract
            .new_order_created_filter()
            .from_block(U64::from(height))
            .query_with_meta()
            .await
            .unwrap();

        //过滤当前所在的market_id的服务引擎
        let new_orders2 = new_orders
            .iter()
            .filter(|(event,_)| event.base_token == base_token && event.quote_token == quote_token)
            .map(|(event,meta_data)| {
                let now = Local::now().timestamp_millis() as u64;
                let order_json = format!("{}{}", serde_json::to_string(&event).unwrap(), now);
                let order_id = sha256(order_json);
                let side = match event.side {
                    true => order::Side::Buy,
                    false => order::Side::Sell,
                };
                let account = format!("{:?}", event.order_user);
                let transaction_hash = format!("{:?}", meta_data.transaction_hash);
                let hash_data_str = u8_arr_to_str(event.hash_data);
                ChainNewOrder {
                    id: order_id,
                    transaction_hash,
                    account,
                    index: event.order_index.as_u32(),
                    num_power: event.num_power.as_u32(),
                    hash_data: hash_data_str.clone(),
                    side,
                    price: event.limit_price,
                    amount: event.order_amount,
                }
            })
            .collect::<Vec<ChainNewOrder>>();
        Ok(new_orders2)
    }
}
