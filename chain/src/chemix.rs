use ethers::prelude::*;
use std::convert::TryFrom;
use std::ops::{Div, Mul};
use std::str::FromStr;
use std::sync::Arc;
use chemix_utils::math::MathOperation;
use crate::k256::ecdsa::SigningKey;
use anyhow::Result;
use chrono::Local;
use chemix_utils::algorithm::sha256;
use chemix_models::order::Side::*;
use chemix_models::order::BookOrder;

abigen!(
    ChemixMain,
    "../contract/ChemixMain.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    ChemixStorage,
    "../contract/ChemixStorage.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub struct ChemixContractClient {
    client: Arc<SignerMiddleware<Provider<Http>,Wallet<SigningKey>>>,
    contract_addr: H160
}

impl ChemixContractClient {
    pub fn new(pri_key:&str,contract_address:&str) -> ChemixContractClient {
        let host = "http://58.33.12.252:8548";
        let provider_http = Provider::<Http>::try_from(host).unwrap();
        let wallet = pri_key
            .parse::<LocalWallet>()
            .unwrap().with_chain_id(15u64);
        let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
        let client = Arc::new(client);
        let contract_addr = Address::from_str(contract_address).unwrap();
        //let test1 : chemixmain_mod::ChemixMain<SignerMiddleware<Middleware, Signer>>= ChemixMain::new(contract_addr, client.clone());
        ChemixContractClient {
            client,
            contract_addr
        }
    }
    pub async fn new_order(&self,side: &str,quoteToken: &str,baseToken : &str,price: f64,amount: f64) -> Result<()>{
       let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let  tokenADecimal  = U256::from(10u128).pow(U256::from(3u32)); //11 -8
        let  tokenBDecimal  = U256::from(10u128).pow(U256::from(14u32)); //22 -8

        let  tmpDecimal  = U256::from(10u128).pow(U256::from(11u32)); //11 -8


        let quoteToken = Address::from_str(quoteToken).unwrap();
        let baseToken = Address::from_str(baseToken).unwrap();

        let amount = U256::from(amount.to_nano()).mul(tokenADecimal);
        let price = U256::from(price.to_nano()).mul(tokenBDecimal).div(tmpDecimal);
        match side {
            "buy" => {
                let result = contract.new_limit_buy_order(quoteToken,baseToken,price,amount)
                    .legacy().send().await?.await?;
                info!("new buy order result  {:?}",result);
            },
            "sell" =>{
                let result = contract.new_limit_sell_order(quoteToken,baseToken,price,amount)
                    .legacy().send().await?.await?;
                info!("new sell order result  {:?}",result);
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }
    pub async fn cancel_order(){
        todo!()
    }

    //fixme:更合适的区分两份合约
    pub async fn filter_new_order_event(&self,height: U64) -> Result<Vec<BookOrder>>{
        let contract = ChemixStorage::new(self.contract_addr, self.client.clone());
        let new_orders: Vec<NewOrderCreatedFilter> = contract
            .new_order_created_filter()
            .from_block(height)
            .query()
            .await
            .unwrap();
        info!(" new_order_created_filter len {} at height {}",new_orders.len(),height);

        let new_orders2 = new_orders
            .iter()
            .map(|x| {
                let now = Local::now().timestamp_millis() as u64;
                let order_json = format!(
                    "{}{}",
                    serde_json::to_string(&x).unwrap(),
                    now
                );
                let order_id = sha256(order_json);
                let side = match x.order_type {
                    true => Buy,
                    false => Sell,
                };
                BookOrder {
                    id: order_id,
                    account: x.order_user.to_string(),
                    side,
                    price: x.limit_price,
                    amount: x.order_amount,
                    created_at: now,
                }
            })
            .collect::<Vec<BookOrder>>();
        Ok(new_orders2)
    }
}
