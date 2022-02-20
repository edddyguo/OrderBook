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

        let mut query = format!("insert into chemix_orders values(");
        for i in 0..order_info.len() {
            if i < order_info.len() - 1 {
                query = format!("{}{},", query, order_info[i]);
            } else {
                query = format!("{}{})", query, order_info[i]);
            }
        }
        info!("insert order successful insert,sql={}", query);
        let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
        // let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &tradesArr[0..tradesArr.len()]);
        if let Err(_err) = result {
            //info!("insert order sql={} failed {:?}", query, err);
            if !crate::restartDB() {
                return;
            }
            result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
        }
        let _rows = result.unwrap();
    }
}

pub fn update_order(order: &UpdateOrder) {
    // fixme:考虑数据后期增加的问题，做每日的临时表
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
    let mut result = crate::CLIENTDB.lock().unwrap().execute(&*sql, &[]);
    if let Err(err) = result {
        info!("update order failed {:?},sql={}", err, sql);
        if !crate::restartDB() {
            return;
        }
        result = crate::CLIENTDB.lock().unwrap().execute(&*sql, &[]);
    }
    info!("success update {} rows", result.unwrap());
    return;
}

pub fn list_available_orders(market_id: &str, side: Side) -> Vec<EngineOrder> {
    let sql = format!("select id,\
    account,
    price,\
    available_amount,\
    side,\
    cast(created_at as text) from chemix_orders \
    where market_id='{}' and available_amount!=\'0\' and side='{}' order by created_at ASC", market_id, side.as_str());
    let mut orders: Vec<EngineOrder> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    if let Err(_err) = result {
        //info!("list_available_orders failed {:?}", err);
        if !crate::restartDB() {
            return orders;
        }
        result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    }
    let rows = result.unwrap();
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
        "select id,market_id,account,side,
         amount,\
         price,\
         status,\
         available_amount,\
         matched_amount,\
         canceled_amount,\
         cast(updated_at as text) ,\
         cast(created_at as text) \
         from chemix_orders where id=$1"
    );
    let mut result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[&id]);
    if let Err(_err) = result {
        //info!("get order failed {:?},sql={}", err, sql);
        if !crate::restartDB() {
            return Err("psql restart failed".to_string());
        }
        result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[&id]);
    }


    let rows = result.unwrap();
    let order = OrderInfo {
        id: rows[0].get(0),
        market_id: rows[0].get(1),
        account: rows[0].get(2),
        side: rows[0].get(3),
        price: U256::from_str_radix(rows[0].get::<usize,&str>(4usize),10).unwrap(),
        amount: U256::from_str_radix(rows[0].get::<usize,&str>(5usize),10).unwrap(),
        status: Status::from(rows[0].get::<usize,&str>(6usize)),
        available_amount: U256::from_str_radix(rows[0].get::<usize,&str>(7usize),10).unwrap(),
        matched_amount: U256::from_str_radix(rows[0].get::<usize,&str>(8usize),10).unwrap(),
        canceled_amount: U256::from_str_radix(rows[0].get::<usize,&str>(9usize),10).unwrap(),
        updated_at: rows[0].get(10),
        created_at: rows[0].get(11),
    };
    Ok(order)
}
