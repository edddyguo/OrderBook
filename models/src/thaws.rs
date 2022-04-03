extern crate rustc_serialize;

use ethers_core::types::U256;

use serde::Deserialize;

//#[derive(Serialize)]
use crate::{assembly_insert_values, struct2array};

use common::utils::time::get_current_time;

use common::types::order::Side as OrderSide;
use common::types::thaw::Status as ThawStatus;

#[derive(Clone, Debug)]
pub enum ThawsFilter<'a> {
    //market,account
    NotConfirmed(&'a str, &'a str),
    Status(ThawStatus),
    //hashdata,block_height
    DelayConfirm(&'a str, u32),
    LastPushed,
}

impl ThawsFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            ThawsFilter::NotConfirmed(market_id, account) => {
                format!("where market_id='{}' and account='{}' and (status='pending' or status='launched')  order by created_at ASC",market_id,account)
            }
            ThawsFilter::Status(status) => {
                format!("where status='{}' order by created_at ASC", status.as_str())
            }
            ThawsFilter::DelayConfirm(hash, height) => {
                format!(
                    " where status='launched' and thaws_hash='{}' and block_height='{}' ",
                    hash, height
                )
            }
            ThawsFilter::LastPushed => {
                "where status='confirmed' order by created_at DESC limit 1".to_string()
            }
        };
        filter_str
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ThawsPO {
    pub order_id: String,
    pub account: String,
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

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct UpdateThaw<'a> {
    pub order_id: String,
    pub cancel_id: String,
    pub block_height: u32,
    pub transaction_hash: String,
    pub status: ThawStatus,
    pub updated_at: &'a str,
}

//todo:考虑没有返回hash但是交易成功的情况？
//todo: 和orders同步的时候做事务的一致性
impl ThawsPO {
    pub fn new(
        order_id: &str,
        account: &str,
        market_id: &str,
        amount: U256,
        price: U256,
        side: OrderSide,
    ) -> ThawsPO {
        ThawsPO {
            order_id: order_id.to_owned(),
            account: account.to_owned(),
            market_id: market_id.to_owned(),
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

pub fn update_thaws(thaws: &Vec<UpdateThaw>) {
    let mut lines_str = "".to_string();
    for thaw in thaws {
        let mut line_str = format!(
            "('{}',{},'{}','{}',cast('{}' as timestamp),'{}')",
            thaw.status.as_str(),
            thaw.block_height,
            thaw.transaction_hash,
            thaw.cancel_id,
            thaw.updated_at,
            thaw.order_id
        );
        if *thaw != *thaws.last().unwrap() {
            line_str += ",";
        }
        lines_str += &line_str;
    }

    let sql = format!(
        "UPDATE chemix_thaws SET (status,block_height,transaction_hash,thaws_hash,updated_at)\
        =(tmp.status,tmp.block_height,tmp.transaction_hash,tmp.thaws_hash,tmp.updated_at) from \
        (values {} ) as tmp (status,block_height,transaction_hash,thaws_hash,updated_at,order_id) where chemix_thaws.order_id=tmp.order_id",lines_str);

    info!("start update thaws {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update thaws {} rows", execute_res);
}

pub fn insert_thaws(thaw_info: &Vec<ThawsPO>) {
    let mut sql = "insert into chemix_thaws values(".to_string();
    let thaws_arr: Vec<Vec<String>> = thaw_info
        .iter()
        .map(|x| struct2array(x))
        .collect::<Vec<Vec<String>>>();

    let values = assembly_insert_values(thaws_arr);
    sql += &values;

    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert {} rows", execute_res);
}

pub fn list_thaws(filter: ThawsFilter) -> Vec<ThawsPO> {
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
    let mut thaws = Vec::<ThawsPO>::new();
    info!("list_thaws sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let info = ThawsPO {
            order_id: row.get::<usize, &str>(0).to_string(),
            account: row.get::<usize, &str>(1).to_string(),
            market_id: row.get::<usize, &str>(2).to_string(),
            transaction_hash: row.get::<usize, &str>(3).to_string(),
            block_height: row.get::<usize, i32>(4),
            thaws_hash: row.get::<usize, &str>(5).to_string(),
            side: row.get::<usize, &str>(6).into(),
            status: row.get::<usize, &str>(7).into(),
            amount: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            price: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            updated_at: row.get::<usize, &str>(10).to_string(),
            created_at: row.get::<usize, &str>(11).to_string(),
        };
        thaws.push(info);
    }
    thaws
}
