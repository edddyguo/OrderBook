use crate::{assembly_insert_values, struct2array, TimeScope, TradeInfoPO};
use common::utils::algorithm::sha256;
use common::utils::time::get_current_time;
use ethers_core::types::U256;
use serde::{Deserialize, Serialize};

extern crate rustc_serialize;
use common::types::order::Side as OrderSide;
use common::types::trade::Status as TradeStatus;
use common::utils::math::U256_ZERO;

#[derive(Clone, Debug)]
pub enum InvalidTradeFilter<'a> {
    ById(&'a str),
    Height(u32),
}
impl InvalidTradeFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            InvalidTradeFilter::ById(id) => {
                format!("where id='{}'", id)
            }
            InvalidTradeFilter::Height(height) => {
                format!(" where block_height='{}' ", height)
            }
        };
        filter_str
    }
}


pub fn insert_invalid_trades(trades: &mut Vec<TradeInfoPO>) {
    info!("start insert info {:#?}", trades);
    if trades.is_empty() {
        return;
    }
    let mut sql = "insert into chemix_trades values(".to_string();
    let trades_arr: Vec<Vec<String>> = trades
        .iter()
        .map(|x| struct2array(x))
        .collect::<Vec<Vec<String>>>();

    let values = assembly_insert_values(trades_arr);
    sql += &values;

    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert traders {} rows", execute_res);
}

pub fn list_invalid_trades(filter: InvalidTradeFilter) -> Vec<TradeInfoPO> {
    let sql = format!(
        "select \
    id,\
    block_height,\
    transaction_hash,\
    hash_data,\
    status,\
    market_id,\
    taker,\
    maker,\
    price,\
    amount,\
    taker_side,\
    maker_order_id, \
    taker_order_id,\
    cast(created_at as text), \
    cast(updated_at as text) \
    from chemix_trades {}",
        filter.to_string()
    );
    let mut trades: Vec<TradeInfoPO> = Vec::new();
    info!("list_trades_sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        //side要结合是taker还是marker来判断
        let info = TradeInfoPO {
            id: row.get(0),
            block_height: row.get(1),
            transaction_hash: row.get(2),
            hash_data: row.get(3),
            status: row.get::<usize, &str>(4usize).into(), //row.get(3),
            market_id: row.get(5),
            taker: row.get(6),
            maker: row.get(7),
            price: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            taker_side: row.get::<usize, &str>(10).into(),
            maker_order_id: row.get(11),
            taker_order_id: row.get(12),
            updated_at: row.get(13),
            created_at: row.get(14),
        };
        trades.push(info);
    }
    trades
}

pub fn delete_invalid_trades(filter: InvalidTradeFilter) {
    todo!()
}
