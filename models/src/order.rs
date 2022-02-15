
use postgres::{config::Config, error::Error, row::SimpleQueryRow, Client, NoTls};

extern crate rustc_serialize;
use serde::Deserialize;


use rustc_serialize::json;

use std::ops::Mul;
//#[derive(Serialize)]
use serde::Serialize;
use slog::info;
use utils::math::narrow;
use utils::time::get_current_time;
use crate::struct2array;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct UpdateOrder {
    pub id: String,
    pub status: String,
    pub available_amount: f64,
    pub canceled_amount: f64,
    pub matched_amount: f64,
    pub updated_at: String,
}

#[derive(Deserialize, RustcDecodable, RustcEncodable, Debug, Default, Clone)]
pub struct EngineOrder {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub side: String,
    pub created_at: String,
}

#[derive(RustcEncodable,Deserialize, Debug,PartialEq,Clone,Serialize)]
pub enum Side {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

/**
amount = available_amount + matched_amount + canceled_amount
*/
#[derive(Deserialize, Debug, Clone,Serialize,Default)]
pub struct OrderInfo {
    pub id: String,
    pub market_id: String,
    pub account: String,
    pub side: String,
    pub price: f64,
    pub amount: f64,
    pub status: String,
    pub available_amount: f64,
    pub matched_amount: f64,
    pub canceled_amount: f64,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct MarketVolume {
    pub marketID: String,
    pub volume: f64,
}

impl OrderInfo {
    pub fn new(id:String, market_id:String, account:String, side: Side, price:u64, amount:u64) -> OrderInfo {
        let side = match side {
            Side::Buy => {"buy"}
            Side::Sell => {"sell"}
        };
        OrderInfo {
            id,
            market_id,
            account,
            side: side.to_string(),
            price: narrow(price),
            amount: narrow(amount),
            status: "pending".to_string(),
            available_amount: narrow(amount),
            matched_amount: 0.0,
            canceled_amount: 0.0,
            updated_at: get_current_time(),
            created_at: get_current_time()
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
        // info!("insert order successful insert,sql={}", query);
        let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
        // let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &tradesArr[0..tradesArr.len()]);
        if let Err(err) = result {
            //info!("insert order sql={} failed {:?}", query, err);
            if !crate::restartDB() {
                return;
            }
            result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
        }
        let rows = result.unwrap();
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
        //info!("update order failed {:?},sql={}", err, sql);
        if !crate::restartDB() {
            return;
        }
        result = crate::CLIENTDB.lock().unwrap().execute(&*sql, &[]);
    }
    // info!("success update {} rows", result.unwrap());
    return;
}

pub fn list_available_orders(market_id: &str,side: &str, channel: &str) -> Vec<EngineOrder> {
    let sort_by = if side == "buy" {
        "DESC"
    }else {
        "ASC"
    };

    let sql = format!("select id,cast(price as float8),cast(available_amount as float8),side,cast(created_at as text) from chemix_orders \
    where market_id='{}' and available_amount>0 and side='{}' order by price {} ,created_at ASC", market_id, side, sort_by);
    let mut orders: Vec<EngineOrder> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    if let Err(err) = result {
        //info!("list_available_orders failed {:?}", err);
        if !crate::restartDB() {
            return orders;
        }
        result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    }
    let rows = result.unwrap();
    for row in rows {
        let info = EngineOrder {
            id: row.get(0),
            price: row.get(1),
            amount: row.get(2),
            side: row.get(3),
            created_at: row.get(4),
        };
        orders.push(info);
    }
    orders
}

pub fn get_order(id: &str) -> OrderInfo {
    let sql = format!(
        "select id,market_id,account,side,
         cast(amount as float8),\
         cast(price as float8),\
         status,\
         cast(available_amount as float8),\
         cast(matched_amount as float8),\
         cast(canceled_amount as float8),\
         cast(updated_at as text) ,\
         cast(created_at as text) \
         from chemix_orders where id=$1"
    );
    let mut order: OrderInfo = Default::default();
    let mut result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[&id]);
    if let Err(err) = result {
        //info!("get order failed {:?},sql={}", err, sql);
        if !crate::restartDB() {
            return order;
        }
        result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[&id]);
    }
    //id 唯一，直接去第一个成员
    /***
    status: rows[0].get(2),
        amount: rows[0].get(3),
        available_amount: rows[0].get(4),
        canceled_amount: rows[0].get(6),
        updated_at: rows[0].get(8),
    */
    let rows = result.unwrap();
    order = OrderInfo {
        id: rows[0].get(0),
        market_id: rows[0].get(1),
        account: rows[0].get(2),
        side: rows[0].get(3),
        price: rows[0].get(4),
        amount: rows[0].get(5),
        status: rows[0].get(6),
        available_amount: rows[0].get(7),
        matched_amount: rows[0].get(8),
        canceled_amount: rows[0].get(9),
        updated_at: rows[0].get(10),
        created_at: rows[0].get(11)
    };
    order
}