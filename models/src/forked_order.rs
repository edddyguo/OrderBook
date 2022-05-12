extern crate rustc_serialize;

use ethers_core::types::U256;

use serde::Deserialize;

//#[derive(Serialize)]
use crate::{assembly_insert_values, struct2array, TimeScope};
use serde::Serialize;

use common::utils::time::get_current_time;

use common::types::*;

use common::types::order::Status as OrderStatus;

use anyhow::Result;
use common::types::order::Side as OrderSide;
use common::utils::math::U256_ZERO;
use crate::order::{OrderFilter, OrderInfoPO, UpdateOrder};


#[derive(Clone, Debug)]
pub enum ForkedOrderFilter<'a> {
    ById(&'a str),
    ByHeight(u32),
}

impl ForkedOrderFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            ForkedOrderFilter::ByHeight(height) => {
                format!("where block_height='{}'", height)
            }
            ForkedOrderFilter::ById(id) => {
                format!("where id='{}'", id)
            }
        };
        filter_str
    }
}

pub fn insert_forked_orders(orders: &Vec<OrderInfoPO>) {
    //todo 这个后边的括号可以挪走
    let mut sql = "insert into chemix_forked_orders values(".to_string();
    let orders_arr: Vec<Vec<String>> = orders
        .iter()
        .map(|x| struct2array(x))
        .collect::<Vec<Vec<String>>>();

    let values = assembly_insert_values(orders_arr);
    sql += &values;

    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert forked_orders {} rows", execute_res);
}

pub fn update_forked_orders(orders: &Vec<UpdateOrder>) {
    let mut lines_str = "".to_string();
    for order in orders {
        let mut line_str = format!(
            "({},{},{},'{}',cast('{}' as timestamp),'{}')",
            order.available_amount,
            order.canceled_amount,
            order.matched_amount,
            order.status.as_str(),
            order.updated_at,
            order.id
        );
        if *order != *orders.last().unwrap() {
            line_str += ",";
        }
        lines_str += &line_str;
    }

    let sql = format!(
        "UPDATE chemix_orders SET (available_amount,canceled_amount,matched_amount,status,updated_at)\
        =(tmp.available_amount,tmp.canceled_amount,tmp.matched_amount,tmp.status,tmp.updated_at) from \
        (values {} ) as tmp (available_amount,canceled_amount,matched_amount,status,updated_at,id) where chemix_forked_orders.id=tmp.id",lines_str);

    info!("start update orders {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update orders {} rows", execute_res);
}

pub fn list_forked_orders(filter: ForkedOrderFilter) -> Result<Vec<OrderInfoPO>> {
    let sql = format!(
        "select id,index,transaction_hash,block_height,hash_data,market_id,account,side,
         price,\
         amount,\
         status,\
         available_amount,\
         matched_amount,\
         canceled_amount,\
         cast(updated_at as text) ,\
         cast(created_at as text)  from chemix_forked_orders {}",
        filter.to_string()
    );
    info!("list_users_orders2 raw sql {}", sql);
    let mut orders = Vec::<OrderInfoPO>::new();
    let rows = crate::query(sql.as_str())?;
    for row in rows {
        let info = OrderInfoPO {
            id: row.get(0),
            index: row.get::<usize, i32>(1) as u32,
            transaction_hash: row.get(2),
            block_height: row.get::<usize, i32>(3) as u32,
            hash_data: row.get(4),
            market_id: row.get(5),
            account: row.get(6),
            side: row.get::<usize, &str>(7usize).into(),
            price: U256::from_str_radix(row.get::<usize, &str>(8usize), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(9usize), 10).unwrap(),
            status: row.get::<usize, &str>(10usize).into(),
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

pub fn delete_forked_orders(filter: ForkedOrderFilter) {
    todo!()
}
