/**
 将内存和数据库的撮合状态和链余额状态进行剥离
当前模块负责同步
 */
extern crate rustc_serialize;

use std::str::FromStr;
use ethers_core::types::U256;
use jsonrpc_http_server::tokio::prelude::future::Ok;
use serde::Deserialize;

//#[derive(Serialize)]
use serde::Serialize;
use crate::struct2array;
use common::utils::math::narrow;
use common::utils::time::get_current_time;
use std::fmt::Display;
use ethers_core::abi::Address;
use common::types::*;

use common::types::order::Status as OrderStatus;
use common::types::trade::Status as TradeStatus;
use common::types::order::Side as OrderSide;


#[derive(Deserialize, Debug, Clone)]
pub struct Thaws{
    pub order_id: String,
    pub account: Address,
    pub market_id: String,
    pub transaction_hash: String,
    pub block_height: i32,
    pub thaws_hash: String,
    pub side: OrderSide,
    pub status: String, //pending,launch,confirm,abandoned
    pub amount: U256,
    pub price: U256,
    pub updated_at: String,
    pub created_at: String,
}


#[derive(Deserialize, Debug, Clone)]
pub struct UpdateOrder {
    pub id: String,
    pub status: OrderStatus,
    pub available_amount: U256,
    pub canceled_amount: U256,
    pub matched_amount: U256,
    pub updated_at: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EngineOrder {
    pub id: String,
    pub account: String,
    pub price: U256,
    pub amount: U256,
    pub side: OrderSide,
    pub created_at: String,
}


#[derive(Deserialize, Debug, Clone)]
pub struct EngineOrderTmp1 {
    pub id: String,
    pub index: U256,
    pub account: String,
    pub price: U256,
    pub amount: U256,
    pub side: OrderSide,
    pub status: OrderStatus,
    pub created_at: String,
}

#[derive(Deserialize, Debug, Clone,Serialize)]
pub struct EngineOrderTmp2 {
    pub id: String,
    pub index: String,
    pub account: String,
    pub price: f64,
    pub amount: f64,
    pub side: OrderSide,
    pub status: String,
    pub created_at: String,
}



#[derive(Clone, Serialize, Debug)]
pub struct BookOrder {
    pub id: String,
    pub account: String,
    pub index: U256,
    pub side: OrderSide,
    pub price: U256,
    pub amount: U256,
    pub created_at: u64,
}




/**
amount = available_amount + matched_amount + canceled_amount
*/
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct OrderInfo {
    pub id: String,
    pub index: U256,
    pub market_id: String,
    pub account: String,
    pub side: OrderSide,
    pub price: U256,
    pub amount: U256,
    pub status: OrderStatus,
    pub available_amount: U256,
    pub matched_amount: U256,
    pub canceled_amount: U256,
    pub updated_at: String,
    pub created_at: String,
}

//todo:考虑没有返回hash但是交易成功的情况？
//todo: 和orders同步的时候做事务的一致性
impl OrderInfo {
    pub fn new(
        order_id: String,
        account: Address,
        market_id: String,
        transaction_hash: String,
        block_height: u32,
        thaws_hash: String,
        amount: U256,
        price: U256,
        side: OrderSide,
    ) -> Thaws {
        Thaws {
            order_id,
            account,
            market_id,
            transaction_hash,
            block_height: block_height as i32,
            thaws_hash,
            side,
            status: "pending".to_string(),
            amount,
            price,
            updated_at: get_current_time(),
            created_at: get_current_time(),
        }
    }
}

pub fn update_thaws(thaws_hash: String) {
  todo!()
}

pub fn insert_thaws(data: Vec<Thaws>){
    todo!()
}

pub fn list_thaws(status: String) -> Vec<Thaws>{
    todo!()
}