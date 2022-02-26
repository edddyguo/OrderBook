use ethers::prelude::*;
use std::convert::TryFrom;
use std::ops::{Div, Mul};
use std::str::FromStr;
use std::sync::Arc;
use chemix_utils::math::MathOperation;
use crate::k256::ecdsa::SigningKey;
use anyhow::Result;
use chrono::Local;
use chemix_utils::algorithm::{sha256, sha2562};
use chemix_models::order::Side::*;
use chemix_models::order::BookOrder;
use ethers::types::Address;
use chemix_utils::env;
use chemix_utils::env::CONF;
use serde::Serialize;


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

abigen!(
    Vault,
    "../contract/Vault.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[derive(Clone)]
pub struct ChemixContractClient {
    client: Arc<SignerMiddleware<Provider<Http>,Wallet<SigningKey>>>,
    contract_addr: H160,
    pub last_index: Option<U256>,
    pub last_hash_data: Option<[u8; 32]>
}

pub struct VaultClient {
    client: Arc<SignerMiddleware<Provider<Http>,Wallet<SigningKey>>>,
    contract_addr: H160,
    pub last_index: Option<U256>,
    pub last_hash_data: Option<[u8; 32]>
}

#[derive(Clone,Debug)]
pub struct SettleValues2 {
    pub user : Address,
    pub positiveOrNegative1: bool,
    pub incomeBaseToken: U256,
    pub positiveOrNegative2 : bool,
    pub incomeQuoteToken: U256,

}

#[derive(Clone,Debug)]
pub struct CancelOrderState2 {
    pub base_token : Address,
    pub quote_token: Address,
    pub order_user: Address,
    pub cancel_index : U256,
    pub order_index: U256,
    pub hash_data: [u8; 32],
}

#[derive(Clone,Debug,Serialize)]
pub struct ThawBalances {
    pub token : Address,
    pub from: Address,
    pub amount: U256,
}

/****

        emit ThawBalance(token, from, amount);


struct CancelOrderState {
    address   baseToken;
    address   quoteToken;
    address   orderUser;
    uint256   mCancelIndex;
    uint256   orderIndex;
    bytes32   hashData;
}

emit NewCancelOrderCreated(baseToken, quoteToken, newHashData, orderUser,
index, orderIndex);

 */

impl ChemixContractClient {
    pub fn new(pri_key:&str,contract_address:&str) -> ChemixContractClient {
        let chain_rpc = env::CONF.chain_rpc.to_owned();
        let chain_id = env::CONF.chain_id.to_owned();
        let chain_id = chain_id.unwrap().into_string().unwrap().parse::<u64>().unwrap();
        let provider_http = Provider::<Http>::try_from(chain_rpc.unwrap().to_str().unwrap()).unwrap();
        let wallet = pri_key
            .parse::<LocalWallet>()
            .unwrap().with_chain_id(chain_id);
        let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
        let client = Arc::new(client);
        let contract_addr = Address::from_str(contract_address).unwrap();
        //let test1 : chemixmain_mod::ChemixMain<SignerMiddleware<Middleware, Signer>>= ChemixMain::new(contract_addr, client.clone());
        ChemixContractClient {
            client,
            contract_addr,
            last_index: None,
            last_hash_data: None
        }
    }

    pub async fn new_order(&self,side: &str,baseToken: &str,quoteToken : &str,price: f64,amount: f64) -> Result<()>{
       let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let  tokenADecimal  = U256::from(10u128).pow(U256::from(10u32)); //18 -8
        let  tokenBDecimal  = U256::from(10u128).pow(U256::from(7u32)); //15 -8


        let quoteToken = Address::from_str(quoteToken).unwrap();
        let baseToken = Address::from_str(baseToken).unwrap();

        let amount = U256::from(amount.to_nano()).mul(tokenADecimal);
        let price = U256::from(price.to_nano()).mul(tokenBDecimal);
        match side {
            "buy" => {
                info!("new_limit_buy_order,quoteToken={},baseToken={},price={},amount={}",quoteToken,baseToken,price,amount);
                let result = contract.new_limit_buy_order(baseToken,quoteToken,price,amount,U256::from(18u32))
                    .legacy().send().await?.await?;
               info!("new buy order result  {:?}",result.unwrap().block_number);
            },
            "sell" =>{
                info!("new_limit_sell_order,quoteToken={},baseToken={},price={},amount={}",quoteToken,baseToken,price,amount);
                let result = contract.new_limit_sell_order(baseToken,quoteToken,price,amount,U256::from(18u32))
                    .legacy().send().await?.await?;
                info!("new sell order result  {:?}",result.unwrap().block_number);
            }
            _ => {
                unreachable!()
            }
        }
        Ok(())
    }
    /***
       address   baseToken,
        address   quoteToken,
        uint256   orderIndex
    */
    //重复取消的在后台判断逻辑
    pub async fn cancel_order(&self,baseToken: &str,quoteToken: &str,order_index: u32) -> Result<()>{
        let contract = ChemixMain::new(self.contract_addr, self.client.clone());
        let quoteToken = Address::from_str(quoteToken).unwrap();
        let baseToken = Address::from_str(baseToken).unwrap();

        info!("cancel_order market: {}-{} order_index: {}",baseToken,quoteToken,order_index);
        let result = contract.new_cancel_order(baseToken,quoteToken,U256::from(order_index))
            .legacy().send().await?.await?;
        info!("new sell order result  {:?}",result);
        Ok(())
    }

    pub async fn filter_new_cancel_order_created_event(&mut self,height: U64) -> Result<Vec<CancelOrderState2>>{
        let contract = ChemixStorage::new(self.contract_addr, self.client.clone());
        let canceled_orders: Vec<NewCancelOrderCreatedFilter> = contract
            .new_cancel_order_created_filter()
            .from_block(height)
            .query()
            .await
            .unwrap();
        let new_orders2 = canceled_orders
            .iter()
            .map(|x| {
                CancelOrderState2 {
                    base_token: x.base_token,
                    quote_token: x.quote_token,
                    order_user: x.cancel_user,
                    cancel_index: x.m_cancel_index,
                    order_index: x.order_index,
                    hash_data: x.hash_data
                }
            })
            .collect::<Vec<CancelOrderState2>>();
        Ok(new_orders2)
    }



    pub async fn thaw_balances(&self, users : Vec<ThawBalances>) -> Result<Option<TransactionReceipt>>{
        info!("test1 {:?},{:?}",self.last_index,self.last_hash_data);
        let chemix_vault = CONF.chemix_vault.to_owned();
        let vault_address = Address::from_str(chemix_vault.unwrap().to_str().unwrap()).unwrap();
        let contract = Vault::new(vault_address, self.client.clone());
        let now = Local::now().timestamp_millis() as u64;
        let order_json = format!(
            "{}{}",
            serde_json::to_string(&users).unwrap(),
            now
        );
        let cancel_id = sha2562(order_json);

        let users2 = users.iter().map(|x| {
            ThawInfos {
                token : x.token,
                from: x.from,
                amount: x.amount,
            }
        }).collect::<Vec<ThawInfos>>();

        let result : Option<TransactionReceipt> = contract.thaw_balance(cancel_id,users2).legacy().send().await?.await?;
        info!("thaw_balance res = {:?}",result);
        Ok(result)
    }


    pub async fn settlement_trades(&self, base_token:&str,quote_token:&str,trades : Vec<SettleValues2>) -> Result<Option<TransactionReceipt>>{
        info!("test1 {:?},{:?}",self.last_index,self.last_hash_data);
        let chemix_vault = CONF.chemix_vault.to_owned();
        let contract_addr = Address::from_str(chemix_vault.unwrap().to_str().unwrap()).unwrap();
        let contract = Vault::new(contract_addr, self.client.clone());

        let base_token_address = Address::from_str(base_token).unwrap();
        let quote_token_address = Address::from_str(quote_token).unwrap();
        let trades2 = trades.iter().map(|x|{
            SettleValues {
                user: x.user,
                positive_or_negative_1: x.positiveOrNegative1,
                income_base_token:  x.incomeBaseToken,
                positive_or_negative_2: x.positiveOrNegative2,
                income_quote_token: x.incomeQuoteToken
            }
        }).collect::<Vec<SettleValues>>();
        let result : Option<TransactionReceipt> = contract.settlement(base_token_address,quote_token_address,self.last_index.unwrap(),self.last_hash_data.unwrap(),trades2).legacy().send().await?.await?;
        info!("settlement_trades res = {:?}",result);
        Ok(result)
    }


    //fixme:更合适的区分两份合约
    pub async fn filter_new_order_event(&mut self,height: U64) -> Result<Vec<BookOrder>>{
        let contract = ChemixStorage::new(self.contract_addr, self.client.clone());
        let new_orders: Vec<NewOrderCreatedFilter> = contract
            .new_order_created_filter()
            .from_block(height)
            .query()
            .await
            .unwrap();

        if !new_orders.is_empty() {
            info!(" new_order_created_filter len {:?} at height {},order_user={:?}",new_orders,height,new_orders[0].order_user);
            let last_order = &new_orders[new_orders.len()-1];
            self.last_index = Some(last_order.order_index);
            self.last_hash_data = Some(last_order.hash_data);
        }


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
                let side = match x.side {
                    true => Buy,
                    false => Sell,
                };
                info!("___0001_{}",x.order_user);
                let account = format!("{:?}",x.order_user);
                info!("___0002_{}",account);
                BookOrder {
                    id: order_id,
                    account,
                    index: x.order_index,
                    side,
                    price: x.limit_price,
                    amount: x.order_amount,
                    created_at: now,
                }
            })
            .collect::<Vec<BookOrder>>();
        Ok(new_orders2)
    }

    fn approve(){
        todo!()
    }
}
