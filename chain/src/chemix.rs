use ethers::prelude::*;
use std::convert::TryFrom;
use std::ops::{Div, Mul};
use std::str::FromStr;
use std::sync::Arc;
use chemix_utils::math::MathOperation;
use crate::k256::ecdsa::SigningKey;
use anyhow::Result;


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

pub struct ChemixMainClient{
    client: Arc<SignerMiddleware<Provider<Http>,Wallet<SigningKey>>>,
    contract_addr: H160
}

impl ChemixMainClient {
    pub fn new(pri_key:&str,contract_address:&str) -> ChemixMainClient{
        let host = "http://58.33.12.252:8548";
        let provider_http = Provider::<Http>::try_from(host).unwrap();
        let wallet = pri_key
            .parse::<LocalWallet>()
            .unwrap().with_chain_id(15u64);
        let client = SignerMiddleware::new(provider_http.clone(), wallet.clone());
        let client = Arc::new(client);
        let contract_addr = Address::from_str(contract_address).unwrap();
        //let test1 : chemixmain_mod::ChemixMain<SignerMiddleware<Middleware, Signer>>= ChemixMain::new(contract_addr, client.clone());
        ChemixMainClient {
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
            _ => {}
        }
        Ok(())
    }
}

/***
struct ChemixStorageClient{
    client:
}

impl ChemixStorageClient {
    fn new(){
        todo!()
    }

    fn filter_new_order_event(&self){
        todo!()
    }
}

 */
