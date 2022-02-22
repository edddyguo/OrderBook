extern crate rustc_serialize;

use std::str::FromStr;
use ethers_core::types::U256;
use jsonrpc_http_server::tokio::prelude::future::Ok;
use serde::Deserialize;

//#[derive(Serialize)]
use serde::Serialize;
use crate::struct2array;
use crate::Side::{Buy, Sell};
use chemix_utils::math::narrow;
use chemix_utils::time::get_current_time;
use std::fmt::Display;


#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum Status {
    #[serde(rename = "full_filled")]
    FullFilled,
    #[serde(rename = "partial_filled")]
    PartialFilled,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "abandoned")]
    Abandoned,
}

/***

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Buy => "buy",
            Sell => "sell",
        }
    }
}
*/
impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullFilled => "full_filled",
            Self::Abandoned => "abandoned",
            Self::PartialFilled => "partial_filled",
            Self::Pending => "pending",
            Self::Canceled => "canceled",
        }
    }
}

impl From<&str> for Status {
    fn from(status_str: &str) -> Self {
        match status_str {
            "full_filled" => Self::FullFilled,
            "partial_filled" => Self::PartialFilled,
            "pending" => Self::Pending,
            "canceled" => Self::Canceled,
            "abandoned" => Self::Abandoned,
            _ => unreachable!()
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct UpdateOrder {
    pub id: String,
    pub status: String,
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
    pub side: Side,
    pub created_at: String,
}


#[derive(Deserialize, Debug, Clone)]
pub struct EngineOrderTmp1 {
    pub id: String,
    pub index: U256,
    pub account: String,
    pub price: U256,
    pub amount: U256,
    pub side: Side,
    pub status: Status,
    pub created_at: String,
}

#[derive(Deserialize, Debug, Clone,Serialize)]
pub struct EngineOrderTmp2 {
    pub id: String,
    pub index: String,
    pub account: String,
    pub price: f64,
    pub amount: f64,
    pub side: String,
    pub status: String,
    pub created_at: String,
}

#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum Side {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

#[derive(Clone, Serialize, Debug)]
pub struct BookOrder {
    pub id: String,
    pub account: String,
    pub index: U256,
    pub side: Side,
    pub price: U256,
    pub amount: U256,
    pub created_at: u64,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Buy => "buy",
            Sell => "sell",
        }
    }

    pub fn contrary(&self) -> Side {
        match self {
            Buy => Sell,
            Sell => Buy,
        }
    }
}

impl From<&str> for Side {
    fn from(side_str: &str) -> Self {
        match side_str {
            "buy" => Self::Buy,
            "sell" => Self::Sell,
            _ => unreachable!()
        }
    }
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
    pub side: String,
    pub price: U256,
    pub amount: U256,
    pub status: Status,
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
        market_id: String,
        account: String,
        side: Side,
        price: U256,
        amount: U256,
    ) -> OrderInfo {
        let side = match side {
            Side::Buy => "buy",
            Side::Sell => "sell",
        };
        OrderInfo {
            id,
            index,
            market_id,
            account,
            side: side.to_string(),
            price,
            amount,
            status: Status::Pending,
            available_amount: amount,
            matched_amount: U256::from(0),
            canceled_amount:  U256::from(0),
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
        order.status,
        order.updated_at,
        order.id
    );
    info!("start update order {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update order {} rows", execute_res);
}

pub fn list_available_orders(market_id: &str, side: Side) -> Vec<EngineOrder> {
    let sql = format!("select id,\
    account,
    price,\
    available_amount,\
    side,\
    cast(created_at as text) from chemix_orders \
    where market_id='{}' and available_amount!=\'0\' and side='{}' order by created_at ASC", market_id, side.as_str());
    let mut orders = Vec::<EngineOrder>::new();
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(4);
        let side = match side_str.as_str() {
            "buy" => Buy,
            "sell" => Sell,
            _ => {
                unreachable!()
            }
        };
        let info = EngineOrder {
            id: row.get(0),
            account: row.get(1),
            price: U256::from_str_radix(row.get::<usize,&str>(2),10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize,&str>(3),10).unwrap(),
            side,
            created_at: row.get(5),
        };
        orders.push(info);
    }
    orders.sort_by(|a,b|{
        a.price.partial_cmp(&b.price).unwrap()
    });
    //let sort_by = if side == "buy" { "DESC" } else { "ASC" };
    match side {
        Buy => {orders.reverse()}
        Sell => {}
    };
    orders
}

pub fn get_order(id: &str) -> Result<OrderInfo,String> {
    let sql = format!(
        "select id,index,market_id,account,side,
         amount,\
         price,\
         status,\
         available_amount,\
         matched_amount,\
         canceled_amount,\
         cast(updated_at as text) ,\
         cast(created_at as text) \
         from chemix_orders where id=\'{}\'",id
    );
    let rows = crate::query(sql.as_str()).unwrap();
    let order = OrderInfo {
        id: rows[0].get(0),
        index: U256::from(rows[0].get::<usize,i32>(1)),
        market_id: rows[0].get(2),
        account: rows[0].get(3),
        side: rows[0].get(4),
        price: U256::from_str_radix(rows[0].get::<usize,&str>(5usize),10).unwrap(),
        amount: U256::from_str_radix(rows[0].get::<usize,&str>(6usize),10).unwrap(),
        status: Status::from(rows[0].get::<usize,&str>(7usize)),
        available_amount: U256::from_str_radix(rows[0].get::<usize,&str>(8usize),10).unwrap(),
        matched_amount: U256::from_str_radix(rows[0].get::<usize,&str>(9usize),10).unwrap(),
        canceled_amount: U256::from_str_radix(rows[0].get::<usize,&str>(10usize),10).unwrap(),
        updated_at: rows[0].get(11),
        created_at: rows[0].get(12),
    };
    Ok(order)
}



pub fn list_users_orders(account: &str, status1: Status,status2: Status,limit: u32) -> Vec<EngineOrderTmp1> {
    let sql = format!("select id,index,\
    account,\
    price,\
    available_amount,\
    side,\
    status,\
    cast(created_at as text) from chemix_orders \
    where account='{}' and (status=\'{}\' or status=\'{}\') order by created_at ASC limit {}", account, status1.as_str(),status2.as_str(),limit);
    info!("list_users_orders raw sql {}",sql);
    let mut orders = Vec::<EngineOrderTmp1>::new();
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(5);
        let side = Side::from(side_str.as_str());

        let status_str: String = row.get(6);
        let status = Status::from(status_str.as_str());

        let info = EngineOrderTmp1 {
            id: row.get(0),
            index: U256::from(row.get::<usize,i32>(1)),
            account: row.get(2),
            price: U256::from_str_radix(row.get::<usize,&str>(3),10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize,&str>(4),10).unwrap(),
            side,
            status,
            created_at: row.get(7),
        };
        orders.push(info);
    }
    orders.sort_by(|a,b|{
        a.price.partial_cmp(&b.price).unwrap()
    });
    orders
}