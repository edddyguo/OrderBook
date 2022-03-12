extern crate rustc_serialize;

use ethers_core::types::U256;

use serde::Deserialize;

//#[derive(Serialize)]
use crate::{struct2array, TimeScope};
use serde::Serialize;

use common::utils::time::get_current_time;

use common::types::*;

use common::types::order::Status as OrderStatus;

use common::types::order::Side as OrderSide;
use common::utils::math::U256_ZERO;
use anyhow::Result;

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

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct EngineOrderTmp2 {
    pub id: String,
    pub index: String,
    pub account: String,
    pub price: f64,
    pub amount: f64,
    pub side: OrderSide,
    pub status: String,
    pub created_at: u64,
}

#[derive(Clone, Serialize, Debug)]
pub struct BookOrder {
    pub id: String,
    pub account: String,
    pub side: OrderSide,
    pub price: U256,
    pub amount: U256,
    pub created_at: u64,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct OrderInfo {
    pub id: String,
    pub index: u32,
    pub transaction_hash: String,
    pub block_height: u32,
    pub hash_data: String,
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

#[derive(Deserialize, Debug, Default)]
pub struct MarketVolume {
    pub marketID: String,
    pub volume: f64,
}

#[derive(Clone, Debug)]
pub enum OrderFilter {
    GetLastOne,
    ById(String),
    ByIndex(u32),
    //market_id
    AvailableOrders(String, order::Side),
    //account,status_arr,limit
    UserOrders(String,order::Status,order::Status,u32)
}

impl OrderFilter {
    pub fn to_string(&self) ->  String{
        match self {
            OrderFilter::GetLastOne => {
                "order by index desc limit 1".to_string()
            }
            OrderFilter::ById(id) => {
                let filter_str = format!("where id='{}'",id);
                filter_str
            }
            OrderFilter::ByIndex(index) => {
                let filter_str = format!("where index='{}'",index);
                filter_str
            }
            OrderFilter::AvailableOrders(market_id, side) => {
                let filter_str = format!("where market_id='{}' and available_amount!='0' and side='{}' order by created_at ASC",market_id,side.as_str());
                filter_str
            }
            OrderFilter::UserOrders(account, status1, status2,limit) => {
                let filter_str = format!("where market_id='{}' and (status='{}' or status='{}')  order by created_at ASC limit {}",account,status1.as_str(),status2.as_str(),limit);
                filter_str
            }
        }
    }
}

impl OrderInfo {
    pub fn new(
        id: String,
        index: u32,
        transaction_hash: String,
        block_height: u32,
        hash_data: String,
        market_id: String,
        account: String,
        side: OrderSide,
        price: U256,
        amount: U256,
    ) -> OrderInfo {
        OrderInfo {
            id,
            index,
            transaction_hash,
            block_height,
            hash_data,
            market_id,
            account,
            side,
            price,
            amount,
            status: order::Status::Pending,
            available_amount: amount,
            matched_amount: U256_ZERO,
            canceled_amount: U256_ZERO,
            updated_at: get_current_time(),
            created_at: get_current_time(),
        }
    }
}

pub fn insert_order(orders: Vec<OrderInfo>) {
    //fixme: 想办法批量插入
    for order in orders.into_iter() {
        let order_info = struct2array(&order);

        let mut sql = format!("insert into chemix_orders values(");
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

pub fn update_order(order: &UpdateOrder) {
    // todo:考虑数据后期增加的问题，做每日的临时表
    let sql = format!(
        "UPDATE chemix_orders SET (available_amount,\
         canceled_amount,matched_amount,status,updated_at)=\
         ({},{},{},'{}','{}') WHERE id='{}'",
        order.available_amount,
        order.canceled_amount,
        order.matched_amount,
        order.status.as_str(),
        order.updated_at,
        order.id
    );
    info!("start update order {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update order {} rows", execute_res);
}

pub fn update_order_status(
    status: OrderStatus,
    available_amount: U256,
    canceled_amount: U256,
    order_id: &str,
) {
    // todo:考虑数据后期增加的问题，做每日的临时表
    let sql = format!(
        "UPDATE chemix_orders SET (available_amount,canceled_amount,status,updated_at)=\
         ({},{},'{}','{}') WHERE id='{}'",
        available_amount,
        canceled_amount,
        status.as_str(),
        get_current_time(),
        order_id
    );
    info!("start update order {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update order {} rows", execute_res);
}


pub fn list_orders(filter: OrderFilter) -> Result<Vec<OrderInfo>> {
    let sql = format!(
        "select id,index,transaction_hash,block_height,hash_data,market_id,account,side,
         price,\
         amount,\
         status,\
         available_amount,\
         matched_amount,\
         canceled_amount,\
         cast(updated_at as text) ,\
         cast(created_at as text)  from chemix_orders {}",
        filter.to_string()
    );
    info!("list_users_orders2 raw sql {}", sql);
    let mut orders = Vec::<OrderInfo>::new();
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let info = OrderInfo {
            id: row.get(0),
            index: row.get::<usize, i32>(1) as u32,
            transaction_hash: row.get(2),
            block_height: row.get::<usize, i32>(3) as u32,
            hash_data: row.get(4),
            market_id: row.get(5),
            account: row.get(6),
            side: OrderSide::from(row.get::<usize, &str>(7usize)), //row.get(4),
            price: U256::from_str_radix(row.get::<usize, &str>(8usize), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(9usize), 10).unwrap(),
            status: order::Status::from(row.get::<usize, &str>(10usize)),
            available_amount: U256::from_str_radix(row.get::<usize, &str>(11usize), 10)
                .unwrap(),
            matched_amount: U256::from_str_radix(row.get::<usize, &str>(12usize), 10).unwrap(),
            canceled_amount: U256::from_str_radix(row.get::<usize, &str>(13usize), 10).unwrap(),
            updated_at: row.get(14),
            created_at: row.get(15),
        };
        orders.push(info);
    }
    Ok(orders)
}

//
pub fn get_order_num(scope: TimeScope) -> u32 {
    let scope_str = scope.filter_str();
    let sql = format!(
        "select cast(count(1) as integer) from chemix_orders {} ",
        scope_str
    );
    let rows = crate::query(sql.as_str()).unwrap();
    rows[0].get::<usize, i32>(0) as u32
}
//
pub fn get_order_volume(scope: TimeScope, market_id: &str) -> U256 {
    //select amount from chemix_orders where created_at > NOW() - INTERVAL '7 day' and  market_id='BTC-USDT';
    let filter_str = match scope {
        TimeScope::NoLimit => {
            format!("where market_id='{}' ", market_id)
        }
        TimeScope::SevenDay => {
            format!("{} and market_id='{}' ", scope.filter_str(), market_id)
        }
        TimeScope::OneDay => {
            format!("{} and market_id='{}' ", scope.filter_str(), market_id)
        }
    };
    let sql = format!("select amount from chemix_orders {}", filter_str);
    let mut sum = U256_ZERO;
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        sum += U256::from_str_radix(row.get::<usize, &str>(0), 10).unwrap()
    }
    sum
}

//user num from scope time age to now or no time limit
pub fn get_user_number(scope: TimeScope) -> u32 {
    let scope_str = scope.filter_str();
    let sql =format!("select cast(count(1) as integer) from (select account from chemix_orders {} group by account) as users",scope_str);
    let rows = crate::query(sql.as_str()).unwrap();
    rows[0].get::<usize, i32>(0) as u32
}
