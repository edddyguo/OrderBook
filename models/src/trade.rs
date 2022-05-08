use crate::{assembly_insert_values, struct2array, TimeScope};
use common::utils::algorithm::sha256;
use common::utils::time::get_current_time;
use ethers_core::types::U256;
use postgres::types::ToSql;
use serde::{Deserialize, Serialize};

extern crate rustc_serialize;
use common::types::order::Side as OrderSide;
use common::types::trade::Status as TradeStatus;
use common::utils::math::U256_ZERO;

#[derive(Clone, Debug)]
pub enum TradeFilter<'a> {
    //id
    OrderId(&'a str),
    //market,limit
    MarketId(&'a str, u32),
    //account,market,status,limit
    Recent(&'a str, &'a str, TradeStatus, u32),
    //hashdata,block_height
    DelayConfirm(&'a str, u32),
    //status, limit
    Status(TradeStatus, u32),
    //height
    NotConfirm(u32),
    LastPushed,
    ZeroHeight,
}
impl TradeFilter<'_> {
    pub fn to_string(&self) -> String {
        let filter_str = match self {
            TradeFilter::OrderId(id) => {
                format!("where taker_order_id='{}' or maker_order_id='{}'", id, id)
            }
            TradeFilter::Status(status, limit) => {
                format!("where status='{}' limit {}", status.as_str(), limit)
            }
            TradeFilter::MarketId(market_id, limit) => {
                format!(
                    "where market_id='{}' order by created_at desc  limit {}",
                    market_id, limit
                )
            }
            TradeFilter::Recent(account, market_id, status, limit) => {
                format!(
                    " where (taker='{}' or maker='{}') and market_id='{}' \
                and status='{}' limit {}",
                    account,
                    account,
                    market_id,
                    status.as_str(),
                    limit
                )
            }
            TradeFilter::DelayConfirm(hash, height) => {
                //考虑到http失败但实际上链的情况，在这里将block_height为零的也确认
                format!(
                    " where status='launched' and hash_data='{}' and (block_height='{}' or  block_height=0)",
                    hash, height
                )
            }
            //    NotConfirm(u32),
            TradeFilter::NotConfirm(height) => {
                format!(" where status='launched' and block_height<{}", height)
            }
            TradeFilter::LastPushed => {
                "where status='confirmed' order by created_at desc  limit 1".to_string()
            }
            TradeFilter::ZeroHeight => {
                "where block_height='0' ".to_string()
            }
        };
        filter_str
    }
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct TradeInfoPO {
    pub id: String,
    pub block_height: i32,
    pub transaction_hash: String,
    pub hash_data: String,
    pub status: TradeStatus,
    pub market_id: String,
    pub maker: String,
    pub taker: String,
    pub price: U256,
    pub amount: U256,
    pub taker_side: OrderSide,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub updated_at: String,
    pub created_at: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct UpdateTrade<'a> {
    pub id: String,
    pub status: TradeStatus,
    pub block_height: u32,
    pub transaction_hash: String,
    pub hash_data: String,
    pub updated_at: &'a str,
}

impl TradeInfoPO {
    pub fn new(
        market_id: &str,
        taker: &str,
        maker: &str,
        price: U256,
        amount: U256,
        taker_side: OrderSide,
        maker_order_id: &str,
        taker_order_id: &str,
    ) -> TradeInfoPO {
        let now = get_current_time();
        let mut trade = TradeInfoPO {
            id: "".to_string(),
            block_height: -1,
            transaction_hash: "".to_string(),
            hash_data: "".to_string(),
            status: TradeStatus::Matched,
            market_id: market_id.to_owned(),
            taker: taker.to_owned(),
            maker: maker.to_owned(),
            price,
            amount,
            taker_side,
            maker_order_id: maker_order_id.to_owned(),
            taker_order_id: taker_order_id.to_owned(),
            updated_at: now.clone(),
            created_at: now.clone(),
        };
        trade.id = sha256(format!("{}{}", serde_json::to_string(&trade).unwrap(), now));
        trade
    }
}

pub fn insert_trades(trades: &mut Vec<TradeInfoPO>) {
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

pub fn list_trades(filter: TradeFilter) -> Vec<TradeInfoPO> {
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

/***
pub fn update_trade(
    id: &str,
    status: TradeStatus,
    height: u32,
    transaction_hash: &str,
    hash_data: &str,
) {
    let sql = format!(
        "UPDATE chemix_trades SET (status,block_height,transaction_hash,hash_data,updated_at)=\
         ('{}',{},'{}','{}','{}') WHERE id='{}'",
        status.as_str(),
        height,
        transaction_hash,
        hash_data,
        get_current_time(),
        id
    );
    info!("start update trade {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update trade {} rows", execute_res);
}
 */

pub fn update_trades(trades: &Vec<UpdateTrade>) {
    let mut lines_str = "".to_string();
    for trade in trades {
        let mut line_str = format!(
            "('{}',{},'{}','{}',cast('{}' as timestamp),'{}')",
            trade.status.as_str(),
            trade.block_height,
            trade.transaction_hash,
            trade.hash_data,
            trade.updated_at,
            trade.id
        );
        if *trade != *trades.last().unwrap() {
            line_str += ",";
        }
        lines_str += &line_str;
    }

    let sql = format!(
        "UPDATE chemix_trades SET (status,block_height,transaction_hash,hash_data,updated_at)\
        =(tmp.status,tmp.block_height,tmp.transaction_hash,tmp.hash_data,tmp.updated_at) from \
        (values {} ) as tmp (status,block_height,transaction_hash,hash_data,updated_at,id) where chemix_trades.id=tmp.id",lines_str);

    info!("start update trades {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update trades {} rows", execute_res);
}

pub fn update_trade_by_hash(status: TradeStatus, hash_data: &str, block_height: u32) {
    let sql = format!(
        "UPDATE chemix_trades SET (status,updated_at)=\
         ('{}','{}') WHERE hash_data='{}' and block_height={}",
        status.as_str(),
        get_current_time(),
        hash_data,
        block_height
    );
    info!("start update trade {} ", sql);
    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success update trade {} rows", execute_res);
}

pub fn get_current_price(market_id: &str) -> U256 {
    let sql =format!("select price from chemix_trades where market_id='{}' order by created_at desc limit 1;",market_id);
    let rows = crate::query(sql.as_str()).unwrap();
    U256::from_str_radix(rows[0].get::<usize, &str>(0), 10).unwrap()
}

pub fn get_current_price2(market_id: &str) -> Option<U256> {
    let sql =format!("select price from chemix_trades where market_id='{}' order by created_at desc limit 1;",market_id);
    let rows = crate::query(sql.as_str()).unwrap();
    if rows.is_empty() {
        return Some(U256_ZERO);
    }
    Some(U256::from_str_radix(rows[0].get::<usize, &str>(0), 10).unwrap())
}

pub fn get_trade_volume(scope: TimeScope, market_id: &str) -> U256 {
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
    let sql = format!("select amount from chemix_trades {}", filter_str);
    let rows = crate::query(sql.as_str()).unwrap();

    //sum the volume
    rows.iter().fold(U256_ZERO, |acc, x| {
        acc + U256::from_str_radix(x.get::<usize, &str>(0), 10).unwrap()
    })
}
