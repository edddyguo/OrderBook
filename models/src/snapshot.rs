extern crate rustc_serialize;

use ethers_core::types::U256;
//#[derive(Serialize)]
use serde::Serialize;
use crate::struct2array;

/***
  traders int default 0,
  transactions  int default 0,
  order_volume text default '',
  withdraw text default '',
  trade_volume text default '',
  trading_pairs int default 0,
  cec_price text default '',
  snapshot_time bigint,

*/
#[derive(Serialize, Debug, Default)]
pub struct Snapshot {
    pub traders: i32,
    pub transactions: i32,
    pub order_volume: U256,
    pub withdraw: U256,
    pub trade_volume: U256,
    pub trading_pairs: i32,
    pub cec_price: U256,
    pub snapshot_time: i64
}


pub fn get_snapshot() -> Option<Snapshot> {

    let sql = format!("select traders,transactions,order_volume,withdraw,\
    trade_volume,trading_pairs,cec_price,snapshot_time from chemix_snapshot ");
    let execute_res = crate::query(sql.as_str()).unwrap();
    info!("get_snapshot: raw sql {}", sql);
    if execute_res.is_empty(){
        return None;
    }

    Some(Snapshot {
        traders: execute_res[0].get(0),
        transactions: execute_res[0].get(1),
        order_volume: U256::from_str_radix(execute_res[0].get::<usize, &str>(2), 10).unwrap(),
        withdraw: U256::from_str_radix(execute_res[0].get::<usize, &str>(3), 10).unwrap(),
        trade_volume: U256::from_str_radix(execute_res[0].get::<usize, &str>(4), 10).unwrap(),
        trading_pairs: execute_res[0].get(5),
        cec_price: U256::from_str_radix(execute_res[0].get::<usize, &str>(6), 10).unwrap(),
        snapshot_time: execute_res[0].get(7),
    })
}

pub fn insert_snapshot(data: Snapshot) {
    //fixme: 想办法批量插入
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


/***
pub struct Dash {
    pub cumulative_traders: i32,
    pub cumulative_transactions: i32,
    pub cumulative_tvl: U256,
    pub one_day_traders: i32,
    pub one_day_volume: U256,
    pub one_day_transactions: i32,
    pub one_day_tvl: U256,
    pub trading_pairs: i32,
    pub cec_price: U256,
    pub snapshot_time: i64,
}
*/
/****
pub fn update_dash(dash_info: Dash) {
    let sql = format!(
        "UPDATE chemix_dash SET (cumulative_traders,\
        cumulative_transactions,\
        cumulative_tvl,\
        one_day_traders,\
        one_day_volume,\
        one_day_transactions,\
        one_day_tvl,\
        trading_pairs,\
        cec_price,\
        snapshot_time\
        )=('{}','{}',{},'{}','{}','{}','{}','{}','{}','{}')",
        dash_info.cumulative_traders,
        dash_info.cumulative_transactions,
        dash_info.cumulative_tvl,
        dash_info.one_day_traders,
        dash_info.one_day_volume,
        dash_info.one_day_transactions,
        dash_info.one_day_tvl,
        dash_info.trading_pairs,
        dash_info.cec_price,
        dash_info.snapshot_time
    );
    info!("start update trade {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update trade {} rows", execute_res);
}

 */