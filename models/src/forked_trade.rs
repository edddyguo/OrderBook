use crate::{assembly_insert_values, struct2array, TradeInfoPO};


use ethers_core::types::U256;


extern crate rustc_serialize;




#[derive(Clone, Debug)]
pub enum ForkedTradeFilter<'a> {
    ById(&'a str),
    ByHeight(u32),
}
impl ForkedTradeFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            ForkedTradeFilter::ById(id) => {
                format!("where id='{}'", id)
            }
            ForkedTradeFilter::ByHeight(height) => {
                format!(" where block_height='{}' ", height)
            }
        };
        filter_str
    }
}


pub fn insert_forked_trades(trades: &mut Vec<TradeInfoPO>) {
    info!("start insert info {:#?}", trades);
    if trades.is_empty() {
        return;
    }
    let mut sql = "insert into chemix_forked_trades values(".to_string();
    let trades_arr: Vec<Vec<String>> = trades
        .iter()
        .map(|x| struct2array(x))
        .collect::<Vec<Vec<String>>>();

    let values = assembly_insert_values(trades_arr);
    sql += &values;

    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert traders {} rows", execute_res);
}

pub fn list_forked_trades(filter: ForkedTradeFilter) -> Vec<TradeInfoPO> {
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
    from chemix_forked_trades {}",
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

pub fn delete_forked_trades(_filter: ForkedTradeFilter) {
    todo!()
}
