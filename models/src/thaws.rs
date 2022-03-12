extern crate rustc_serialize;

use ethers_core::types::U256;
use std::str::FromStr;

use serde::Deserialize;

//#[derive(Serialize)]
use crate::struct2array;
use serde::Serialize;

use common::utils::time::get_current_time;

use ethers_core::abi::Address;

use common::types::order::Status as OrderStatus;

use common::types::order::Side as OrderSide;
use common::types::thaw::Status as ThawStatus;


#[derive(Clone, Debug)]
pub enum ThawsFilter {
    //market,account
    NotConfirmed(String, String),
}

impl ThawsFilter {
    pub fn to_string(&self) -> String {
        match self {
            ThawsFilter::NotConfirmed(market_id, account) => {
                let filter_str = format!("where market_id='{}' and account='{}' and (status='pending' or status='launched')  order by created_at ASC",market_id,account);
                filter_str
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Thaws {
    pub order_id: String,
    pub account: Address,
    pub market_id: String,
    pub transaction_hash: String,
    pub block_height: i32,
    pub thaws_hash: String,
    pub side: OrderSide,
    pub status: ThawStatus, //pending,launch,confirm,abandoned
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

//todo:考虑没有返回hash但是交易成功的情况？
//todo: 和orders同步的时候做事务的一致性
impl Thaws {
    pub fn new(
        order_id: String,
        account: Address,
        market_id: String,
        amount: U256,
        price: U256,
        side: OrderSide,
    ) -> Thaws {
        Thaws {
            order_id,
            account,
            market_id,
            transaction_hash: "".to_string(),
            block_height: 0,
            thaws_hash: "".to_string(),
            side,
            status: ThawStatus::Pending,
            amount,
            price,
            updated_at: get_current_time(),
            created_at: get_current_time(),
        }
    }
}

pub fn update_thaws1(
    order_id: &str,
    cancel_id: &str,
    transaction_hash: &str,
    block_height: i32,
    status: ThawStatus,
) {
    let sql = format!(
        "UPDATE chemix_thaws SET (thaws_hash,\
         transaction_hash,block_height,status,updated_at)=\
         ('{}','{}',{},'{}','{}') WHERE order_id='{}'",
        cancel_id,
        transaction_hash,
        block_height,
        status.as_str(),
        get_current_time(),
        order_id
    );
    info!("start update order {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update order {} rows", execute_res);
}

pub fn insert_thaws(thaw_info: Vec<Thaws>) {
    //todo: 批量插入
    for order in thaw_info.into_iter() {
        let order_info = struct2array(&order);
        let mut sql = format!("insert into chemix_thaws values(");
        for i in 0..order_info.len() {
            if i < order_info.len() - 1 {
                sql = format!("{}{},", sql, order_info[i]);
            } else {
                sql = format!("{}{})", sql, order_info[i]);
            }
        }
        info!("insert order successful insert,sql={}", sql);
        let execute_res = crate::execute(sql.as_str()).unwrap();
        info!("success insert {} rows", execute_res);
    }
}

pub fn list_thaws(status: ThawStatus) -> Vec<Thaws> {
    let sql = format!(
        "select order_id,\
    account,\
    market_id,\
    transaction_hash,\
    block_height,\
    thaws_hash,\
    side,\
    status,\
    amount,\
    price,\
    cast(updated_at as text),\
    cast(created_at as text) from chemix_thaws \
    where status='{}' order by created_at DESC",
        status.as_str()
    );
    let mut thaws = Vec::<Thaws>::new();
    info!("list_thaws sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(6);
        let side = OrderSide::from(side_str.as_str());

        let status_str: String = row.get(7);
        let status = ThawStatus::from(status_str.as_str());

        let info = Thaws {
            order_id: row.get::<usize, &str>(0).to_string(),
            account: Address::from_str(row.get::<usize, &str>(1)).unwrap(),
            market_id: row.get::<usize, &str>(2).to_string(),
            transaction_hash: row.get::<usize, &str>(3).to_string(),
            block_height: row.get::<usize, i32>(4),
            thaws_hash: row.get::<usize, &str>(5).to_string(),
            side,
            status,
            amount: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            price: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            updated_at: row.get::<usize, &str>(10).to_string(),
            created_at: row.get::<usize, &str>(11).to_string(),
        };
        thaws.push(info);
    }
    thaws
}

//flag足够，该flag在此时全部launched
pub fn list_thaws2(flag: &str) -> Vec<Thaws> {
    let sql = format!(
        "select order_id,\
    account,\
    market_id,\
    transaction_hash,\
    block_height,\
    thaws_hash,\
    side,\
    status,\
    amount,\
    price,\
    cast(updated_at as text),\
    cast(created_at as text) from chemix_thaws \
    where thaws_hash='{}' order by created_at DESC",
        flag
    );
    let mut thaws = Vec::<Thaws>::new();
    info!("list_thaws sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(6);
        let side = OrderSide::from(side_str.as_str());

        let status_str: String = row.get(7);
        let status = ThawStatus::from(status_str.as_str());

        let info = Thaws {
            order_id: row.get::<usize, &str>(0).to_string(),
            account: Address::from_str(row.get::<usize, &str>(1)).unwrap(),
            market_id: row.get::<usize, &str>(2).to_string(),
            transaction_hash: row.get::<usize, &str>(3).to_string(),
            block_height: row.get::<usize, i32>(4),
            thaws_hash: row.get::<usize, &str>(5).to_string(),
            side,
            status,
            amount: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            price: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            updated_at: row.get::<usize, &str>(10).to_string(),
            created_at: row.get::<usize, &str>(11).to_string(),
        };
        thaws.push(info);
    }
    thaws
}


pub fn list_thaws3(filter: ThawsFilter) -> Vec<Thaws> {
    let sql = format!(
        "select order_id,\
    account,\
    market_id,\
    transaction_hash,\
    block_height,\
    thaws_hash,\
    side,\
    status,\
    amount,\
    price,\
    cast(updated_at as text),\
    cast(created_at as text) from chemix_thaws {}",
        filter.to_string()
    );
    let mut thaws = Vec::<Thaws>::new();
    info!("list_thaws sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(6);
        let side = OrderSide::from(side_str.as_str());

        let status_str: String = row.get(7);
        let status = ThawStatus::from(status_str.as_str());

        let info = Thaws {
            order_id: row.get::<usize, &str>(0).to_string(),
            account: Address::from_str(row.get::<usize, &str>(1)).unwrap(),
            market_id: row.get::<usize, &str>(2).to_string(),
            transaction_hash: row.get::<usize, &str>(3).to_string(),
            block_height: row.get::<usize, i32>(4),
            thaws_hash: row.get::<usize, &str>(5).to_string(),
            side,
            status,
            amount: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            price: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            updated_at: row.get::<usize, &str>(10).to_string(),
            created_at: row.get::<usize, &str>(11).to_string(),
        };
        thaws.push(info);
    }
    thaws
}
