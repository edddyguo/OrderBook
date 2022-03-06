extern crate rustc_serialize;

use ethers_core::types::U256;
use postgres::Row;
//#[derive(Serialize)]
use crate::struct2array;
use serde::Serialize;


#[derive(Serialize, Debug, Default)]
pub struct Snapshot {
    pub traders: i32,
    pub transactions: i32,
    pub order_volume: U256,
    pub withdraw: U256,
    pub trade_volume: U256,
    pub trading_pairs: i32,
    pub cec_price: U256,
    pub snapshot_time: i64,
}

//取当前和一天之前的快照
pub fn get_snapshot() -> Option<(Snapshot, Snapshot)> {
    let sql = format!("select traders,transactions,order_volume,withdraw,\
    trade_volume,trading_pairs,cec_price,snapshot_time from chemix_snapshot order by created_at desc limit 24");
    let execute_res = crate::query(sql.as_str()).unwrap();
    info!("get_snapshot: raw sql {}", sql);
    if execute_res.is_empty() {
        return None;
    }
    let gen_snapshot = |row: &Row| Snapshot {
        traders: row.get(0),
        transactions: row.get(1),
        order_volume: U256::from_str_radix(row.get::<usize, &str>(2), 10).unwrap(),
        withdraw: U256::from_str_radix(row.get::<usize, &str>(3), 10).unwrap(),
        trade_volume: U256::from_str_radix(row.get::<usize, &str>(4), 10).unwrap(),
        trading_pairs: row.get(5),
        cec_price: U256::from_str_radix(row.get::<usize, &str>(6), 10).unwrap(),
        snapshot_time: row.get(7),
    };
    Some((
        gen_snapshot(execute_res.first().unwrap()),
        gen_snapshot(execute_res.last().unwrap()),
    ))
}

pub fn insert_snapshot(data: Snapshot) {
    //todo: 批量插入
    let data_arr = struct2array(&data);
    let mut sql = format!("insert into chemix_snapshot values(");
    for i in 0..data_arr.len() {
        if i < data_arr.len() - 1 {
            sql = format!("{}{},", sql, data_arr[i]);
        } else {
            sql = format!("{}{})", sql, data_arr[i]);
        }
    }
    info!("start insert snapshot ,sql={}", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert {} rows", execute_res);
}
