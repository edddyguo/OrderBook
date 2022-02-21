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
use ethers::types::Address;

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

#[derive(Clone)]
pub struct SettleValues2 {
    pub user : Address,
    pub positiveOrNegative1: bool,
    pub incomeQuoteToken: U256,
    pub positiveOrNegative2 : bool,
    pub incomeBaseToken: U256,
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
            contract_addr,
            last_index: None,
            last_hash_data: None
        }
    }
    pub async fn new_order(&self,side: &str,quoteToken: &str,baseToken : &str,price: f64,amount: f64) -> Result<()>{
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
                let result = contract.new_limit_buy_order(quoteToken,baseToken,price,amount,U256::from(18u32))
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

    pub async fn settlement_trades(&self, trades : Vec<SettleValues2>) -> TransactionReceipt{
        info!("test1 {:?},{:?}",self.last_index,self.last_hash_data);
        let contract_addr = Address::from_str("0x4312e54480D2895c84aB9967CCbA0D87c5Ab2f02").unwrap();
        let contract = Vault::new(contract_addr, self.client.clone());
        let tokenA = Address::from_str("0x18D5034280703EA96e36a50f6178E43565eaDc67").unwrap();
        let tokenB = Address::from_str("0x7E62F80cA349DB398983E2Ee1434425f5B888f42").unwrap();
        /***
        let mut trades  = Vec::new();
        trades.push(vault_mod::SettleValues {
            user: Address::from_str("0x613548d151E096131ece320542d19893C4B8c901").unwrap(),
            positive_or_negative_1: false,
            income_quote_token: U256::from(1i32),
            positive_or_negative_2: false,
            income_base_token: U256::from(1i32)
        });

        ub user : Address,
    pub positiveOrNegative1: bool,
    pub incomeQuoteToken: U256,
    pub positiveOrNegative2 : bool,
    pub incomeBaseToken: U256,

         */
        let trades2 = trades.iter().map(|x|{
            SettleValues {
                user: x.user,
                positive_or_negative_1: x.positiveOrNegative1,
                income_quote_token: x.incomeQuoteToken,
                positive_or_negative_2: x.positiveOrNegative2,
                income_base_token:  x.incomeBaseToken
            }
        }).collect::<Vec<SettleValues>>();
        let result : TransactionReceipt = contract.settlement(tokenA,tokenB,self.last_index.unwrap(),self.last_hash_data.unwrap(),trades2).legacy().send().await.unwrap().await.unwrap().unwrap();
        info!("settlement_trades res = {:?},{:?}",result.transaction_hash,result.block_number);
        result
        /***
         address   quoteToken,
        address   baseToken,
        uint256   largestIndex,
        bytes32   hashData,
        settleValues[] calldata settleInfo

        arr
        struct settleValues {
        address  user;
        bool     positiveOrNegative1;
        uint256  incomeQuoteToken;
        bool     positiveOrNegative2;
        uint256  incomeBaseToken;
    }

        */
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
        // emit NewOrderCreated(quoteToken, baseToken, newHashData, orderUser, orderType,
        //                 index, limitPrice, orderAmount);
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
