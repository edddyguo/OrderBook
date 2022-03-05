use actix_cors::Cors;
use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use std::{env, time};
use std::collections::HashSet;
use std::sync::{Arc, RwLock};
use ethers_core::types::U256;
use anyhow::Result;


use chemix_models::api::{get_markets, get_markets2, list_markets as list_markets2, list_markets, MarketInfo};
use chemix_models::order::{list_available_orders, list_users_orders, EngineOrderTmp2, get_user_number, get_order_num, get_order_volume};
use chemix_models::trade::{get_current_price, get_current_price2, get_trade_volume, list_trades};
use common::utils::time::{get_current_time, get_unix_time, time2unix};

use serde::{Deserialize, Serialize};
use chemix_chain::bsc::Node;
use chemix_chain::chemix::ChemixContractClient;
use chemix_models::snapshot::{insert_snapshot, Snapshot};
use chemix_models::TimeScope;
use chemix_models::TimeScope::TwentyFour;
use chemix_models::tokens::list_tokens;

use common::utils::math::u256_to_f64;

use common::types::order::Side as OrderSide;
use common::types::order::Status as OrderStatus;
use common::types::trade::Status as TradeStatus;
use common::env::CONF as ENV_CONF;



static ONE_HOUR :u64 = 60 * 60;
static TEN_MINS :u64 = 10 * 60;

fn gen_kline(){
    todo!()
}


//get {token}-usdt price
fn get_token_price(quote_symbol: &str) -> Option<U256>{
    if quote_symbol == "USDT" {
        return Some(U256::from(1_000_000_000_000_000i64)); //1U
    }
    let market_id = format!("{}-USDT",quote_symbol);
    //todo:15
    let cec_dicimal = U256::from(10u128).pow(U256::from(15u32));
    match get_markets2(&market_id) {
        None => {
            //必须usdt和cec有一个交易对
            let token2cec_market_id = format!("{}-CEC",quote_symbol);
            let cec_price=  get_current_price2("CEC-USDT").unwrap();
            let token2cec_price=  get_current_price2(token2cec_market_id.as_str()).unwrap();
            Some(token2cec_price * cec_price / cec_dicimal)
        }
        Some(_) => {
            match get_current_price2(&market_id) {
                None => {
                    Some(U256::from(0))
                },
                Some(price) => {
                    Some(price)
                }
            }
        }
    }
}

async fn gen_chemix_profile(vault_client: &ChemixContractClient){
    //todo: 从链上拿
    let mut current_withdraw_value = U256::from(0);
    for token in list_tokens() {
        let base_token_decimal = U256::from(10u128).pow(U256::from(token.base_contract_decimal));
        let price = get_token_price(token.symbol.as_str()).unwrap();
        let withdraw_volume = vault_client.vault_total_withdraw_volume(token.address).await.unwrap();
        let value = withdraw_volume * price / base_token_decimal;
        current_withdraw_value += value;
    }

    let mut total_order_value = U256::from(0);
    let mut total_trade_value = U256::from(0);

    let total_markets = list_markets();
    for market in total_markets.clone() {
        let base_token_decimal = U256::from(10u128).pow(U256::from(market.base_contract_decimal));

        //单个交易对的交易量
        let volume = get_order_volume(TimeScope::NoLimit,&market.id);
        let price = get_token_price(&market.base_token_symbol).unwrap();
        total_order_value += volume * price / base_token_decimal;

        //单个交易对的充值量
        let volume = get_trade_volume(TimeScope::NoLimit,&market.id);
        let price = get_token_price(&market.base_token_symbol).unwrap();
        total_trade_value += volume * price / base_token_decimal;
    }

    let cumulative_transactions = get_order_num(TimeScope::NoLimit) as i32;
    let cumulative_traders = get_user_number(TimeScope::NoLimit) as i32;
    let trading_pairs = total_markets.len() as i32;
    let cec_price = get_token_price("CEC").unwrap();

    let mut current_dash = Snapshot{
        traders: cumulative_traders,
        transactions: cumulative_transactions,
        order_volume: total_order_value,
        withdraw: current_withdraw_value,
        trade_volume: total_trade_value,
        trading_pairs,
        cec_price,
        snapshot_time: get_unix_time() as i64
    };
    insert_snapshot(current_dash);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    //deal market exchange info
    /***
    tokio::spawn(async move {
        loop {
            gen_kline();
        }
    });
     */

    //chemix profile
    info!("0001");

    /**
       let blocking_task = tokio::task::spawn_blocking(|| {
        // This is running on a blocking thread.
        // Blocking here is ok.
    });

    // We can wait for the blocking task like this:
    // If the blocking task panics, the unwrap below will propagate the
    // panic.
    blocking_task.await.unwrap();
     */
    let task1 = tokio::spawn(async move {
        let chemix_vault = ENV_CONF.chemix_vault.to_owned().unwrap();
        //test1
        //let pri_key = "a26660eb5dfaa144ae6da222068de3a865ffe33999604d45bd0167ff1f4e2882";
        let pri_key = "b89da4744ef5efd626df7c557b32f139cdf42414056447bba627d0de76e84c43";

        let chemix_vault_client =
            ChemixContractClient::new(pri_key, chemix_vault.to_str().unwrap());
        loop {
            gen_chemix_profile(&chemix_vault_client).await;
            tokio::time::sleep(time::Duration::from_secs(ONE_HOUR)).await;
        }
    });
    let (task1_res) = tokio::join!(task1);
    info!("0002");
    Ok(())
}
