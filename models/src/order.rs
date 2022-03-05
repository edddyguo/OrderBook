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
    pub created_at: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct BookOrder {
    pub id: String,
    pub account: String,
    pub index: U256,
    pub hash_data: String,
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

impl OrderInfo {
    pub fn new(
        id: String,
        index: U256,
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
            hash_data,
            market_id,
            account,
            side,
            price,
            amount,
            status: order::Status::Pending,
            available_amount: amount,
            matched_amount: U256::from(0),
            canceled_amount: U256::from(0),
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

pub fn list_available_orders(market_id: &str, side: order::Side) -> Vec<EngineOrder> {
    let sql = format!(
        "select id,\
    account,\
    price,\
    available_amount,\
    side,\
    cast(created_at as text) from chemix_orders \
    where market_id='{}' and available_amount!=\'0\' and side='{}' order by created_at ASC",
        market_id,
        side.as_str()
    );
    let mut orders = Vec::<EngineOrder>::new();
    info!("list_available_orders sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(4);
        let side = OrderSide::from(side_str.as_str());
        let info = EngineOrder {
            id: row.get(0),
            account: row.get(1),
            price: U256::from_str_radix(row.get::<usize, &str>(2), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(3), 10).unwrap(),
            side,
            created_at: row.get(5),
        };
        orders.push(info);
    }
    orders
}

#[derive(Clone, Debug, PartialEq)]
/// A Block Hash or Block Number
pub enum IdOrIndex {
    Id(String),
    Index(u32),
}

pub fn get_last_order() -> Result<OrderInfo, String> {
    let sql = format!(
        "select id,index,hash_data,market_id,account,side,
         price,\
         amount,\
         status,\
         available_amount,\
         matched_amount,\
         canceled_amount,\
         cast(updated_at as text) ,\
         cast(created_at as text) \
         from chemix_orders order by index desc limit 1 "
    );
    let rows = crate::query(sql.as_str()).unwrap();
    let order = OrderInfo {
        id: rows[0].get(0),
        index: U256::from(rows[0].get::<usize, i32>(1)),
        hash_data: rows[0].get(2),
        market_id: rows[0].get(3),
        account: rows[0].get(4),
        side: OrderSide::from(rows[0].get::<usize, &str>(5usize)), //rows[0].get(4),
        price: U256::from_str_radix(rows[0].get::<usize, &str>(6usize), 10).unwrap(),
        amount: U256::from_str_radix(rows[0].get::<usize, &str>(7usize), 10).unwrap(),
        status: order::Status::from(rows[0].get::<usize, &str>(8usize)),
        available_amount: U256::from_str_radix(rows[0].get::<usize, &str>(9usize), 10).unwrap(),
        matched_amount: U256::from_str_radix(rows[0].get::<usize, &str>(10usize), 10).unwrap(),
        canceled_amount: U256::from_str_radix(rows[0].get::<usize, &str>(11usize), 10).unwrap(),
        updated_at: rows[0].get(12),
        created_at: rows[0].get(13),
    };
    Ok(order)
}

pub fn get_order<T: Into<IdOrIndex> + Send + Sync>(
    id_or_index: T,
) -> Result<OrderInfo, String> {
    let filter_str = match id_or_index.into() {
        IdOrIndex::Id(id) => {
            format!(" id=\'{}\'", id)
        }
        IdOrIndex::Index(index) => {
            format!(" index=\'{}\'", index)
        }
    };
    let sql = format!(
        "select id,index,hash_data,market_id,account,side,
         price,\
         amount,\
         status,\
         available_amount,\
         matched_amount,\
         canceled_amount,\
         cast(updated_at as text) ,\
         cast(created_at as text) \
         from chemix_orders where {} ",
        filter_str
    );
    let rows = crate::query(sql.as_str()).unwrap();
    let order = OrderInfo {
        id: rows[0].get(0),
        index: U256::from(rows[0].get::<usize, i32>(1)),
        hash_data: rows[0].get(2),
        market_id: rows[0].get(3),
        account: rows[0].get(4),
        side: OrderSide::from(rows[0].get::<usize, &str>(5usize)), //rows[0].get(4),
        price: U256::from_str_radix(rows[0].get::<usize, &str>(6usize), 10).unwrap(),
        amount: U256::from_str_radix(rows[0].get::<usize, &str>(7usize), 10).unwrap(),
        status: order::Status::from(rows[0].get::<usize, &str>(8usize)),
        available_amount: U256::from_str_radix(rows[0].get::<usize, &str>(9usize), 10).unwrap(),
        matched_amount: U256::from_str_radix(rows[0].get::<usize, &str>(10usize), 10).unwrap(),
        canceled_amount: U256::from_str_radix(rows[0].get::<usize, &str>(11usize), 10).unwrap(),
        updated_at: rows[0].get(12),
        created_at: rows[0].get(13),
    };
    Ok(order)
}

pub fn list_users_orders(
    account: &str,
    status1: order::Status,
    status2: order::Status,
    limit: u32,
) -> Vec<EngineOrderTmp1> {
    let sql = format!(
        "select id,index,\
    account,\
    price,\
    available_amount,\
    side,\
    status,\
    cast(created_at as text) from chemix_orders \
    where account='{}' and (status=\'{}\' or status=\'{}\') order by created_at DESC limit {}",
        account,
        status1.as_str(),
        status2.as_str(),
        limit
    );
    info!("list_users_orders raw sql {}", sql);
    let mut orders = Vec::<EngineOrderTmp1>::new();
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(5);
        let side = order::Side::from(side_str.as_str());

        let status_str: String = row.get(6);
        let status = order::Status::from(status_str.as_str());

        let info = EngineOrderTmp1 {
            id: row.get(0),
            index: U256::from(row.get::<usize, i32>(1)),
            account: row.get(2),
            price: U256::from_str_radix(row.get::<usize, &str>(3), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(4), 10).unwrap(),
            side,
            status,
            created_at: row.get(7),
        };
        orders.push(info);
    }
    orders.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
    orders
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
        TimeScope::TwentyFour => {
            format!("{} and market_id='{}' ", scope.filter_str(), market_id)
        }
    };
    let sql = format!("select amount from chemix_orders {}", filter_str);
    let mut sum = U256::from(0);
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
