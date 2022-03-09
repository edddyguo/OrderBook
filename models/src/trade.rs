use crate::{struct2array, TimeScope};
use common::types::*;
use common::utils::algorithm::sha256;
use common::utils::time::get_current_time;
use ethers_core::types::U256;
use serde::{Serialize,Deserialize};

extern crate rustc_serialize;
use common::types::order::Side as OrderSide;
use common::types::trade::Status as TradeStatus;
use common::utils::math::U256_ZERO;

#[derive(Serialize, Debug, Clone,Deserialize)]
pub struct TradeInfo {
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

impl TradeInfo {
    pub fn new(
        market_id: String,
        taker: String,
        maker: String,
        price: U256,
        amount: U256,
        taker_side: OrderSide,
        maker_order_id: String,
        taker_order_id: String,
    ) -> TradeInfo {
        let now = get_current_time();
        let mut trade = TradeInfo {
            id: "".to_string(),
            block_height: 0, //todo: 待加逻辑
            transaction_hash: "".to_string(),
            hash_data: "".to_string(),
            status: TradeStatus::Matched,
            market_id,
            taker,
            maker,
            price,
            amount,
            taker_side,
            maker_order_id,
            taker_order_id,
            updated_at: now.clone(),
            created_at: now.clone(),
        };
        trade.id = sha256(format!("{}{}", serde_json::to_string(&trade).unwrap(), now));
        trade
    }
}

pub fn insert_trades(trades: &mut Vec<TradeInfo>) {
    info!("start insert info {:#?}", trades);
    if trades.is_empty() {
        return;
    }
    let mut sql = format!("insert into chemix_trades values(");
    let tradesArr: Vec<Vec<String>> = trades
        .into_iter()
        .map(|x| struct2array(x))
        .collect::<Vec<Vec<String>>>();
    let mut index = 0;
    let trades_len = tradesArr.len();

    //not used
    let mut tradesArr2: Vec<String> = Default::default();
    // fixme:注入的写法暂时有问题，先直接拼接
    for trade in tradesArr {
        let mut temp_value = "".to_string();
        for i in 0..trade.len() {
            if i < trade.len() - 1 {
                temp_value = format!("{}{},", temp_value, trade[i]);
            } else {
                temp_value = format!("{}{}", temp_value, trade[i]);
            }
        }
        if index < trades_len - 1 {
            sql = format!("{}{}),(", sql, temp_value);
        } else {
            sql = format!("{}{})", sql, temp_value);
        }
        let mut str_trade: Vec<String> = Default::default();
        for item in trade {
            str_trade.push(item);
        }
        tradesArr2.append(&mut str_trade);
        index += 1;
    }

    let execute_res = crate::execute(sql.as_str()).unwrap();
    info!("success insert traders {} rows", execute_res);
}

pub fn list_trades(
    user: Option<String>,
    market_id: Option<String>,
    status: Option<TradeStatus>,
    hash_data: Option<String>,
    block_height: Option<u32>,
    limit: u32,
) -> Vec<TradeInfo> {
    //todo: 待补充场景
    let filter_str = match (user, market_id, status, hash_data,block_height) {
        (None, None, None, None,None) => {
            format!("")
        }
        (Some(account), None, None, None,None) => {
            format!(" where taker='{}' or maker='{}' ", account, account)
        }
        (Some(account), Some(market_id), Some(status), None,None) => {
            format!(
                " where (taker='{}' or maker='{}') and status='{}' and market_id='{}' ",
                account,
                account,
                status.as_str(),
                market_id.as_str()
            )
        }
        (None, Some(id), Some(status), None,None) => {
            format!(
                " where market_id='{}' and status='{}' ",
                id,
                status.as_str()
            )
        }
        (Some(account), Some(id), None, None,None) => {
            format!(
                " where market_id='{}' and (taker='{}' or maker='{}') ",
                id, account, account
            )
        }
        (None, None, Some(status), None,None) => {
            format!(" where status='{}' ", status.as_str())
        }
        (None, None, Some(status), Some(hash_data),Some(height)) => {
            format!(" where status='{}' and hash_data='{}' and block_height={} ", status.as_str(),hash_data.as_str(),height)
        }
        _ => {
            unreachable!()
        }
    };

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
    from chemix_trades \
    {} order by created_at DESC limit {}",
        filter_str, limit
    );
    let mut trades: Vec<TradeInfo> = Vec::new();
    info!("list_trades_sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(10);
        //side要结合是taker还是marker来判断
        let side = order::Side::from(side_str.as_str());
        let info = TradeInfo {
            id: row.get(0),
            block_height: row.get(1),
            transaction_hash: row.get(2),
            hash_data: row.get(3),
            status: TradeStatus::from(row.get::<usize, &str>(4usize)), //row.get(3),
            market_id: row.get(5),
            taker: row.get(6),
            maker: row.get(7),
            price: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            taker_side: side,
            maker_order_id: row.get(11),
            taker_order_id: row.get(12),
            updated_at: row.get(13),
            created_at: row.get(14),
        };
        trades.push(info);
    }
    trades
}

pub fn list_trades2(taker_order_id: &str, hash_data: &str,status: TradeStatus) -> Vec<TradeInfo> {
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
    from chemix_trades \
    where taker_order_id='{}' and hash_data='{}'  and status='{}' order by created_at DESC",
        taker_order_id, hash_data,status.as_str()
    );
    let mut trades: Vec<TradeInfo> = Vec::new();
    info!("list_trades_sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(10);
        let side = order::Side::from(side_str.as_str());
        let info = TradeInfo {
            id: row.get(0),
            block_height: row.get(1), //todo: 待加逻辑
            transaction_hash: row.get(2),
            hash_data: row.get(3),
            status: TradeStatus::from(row.get::<usize, &str>(4usize)), //row.get(3),
            market_id: row.get(5),
            taker: row.get(6),
            maker: row.get(7),
            price: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            taker_side: side,
            maker_order_id: row.get(11),
            taker_order_id: row.get(12),
            updated_at: row.get(13),
            created_at: row.get(14),
        };
        trades.push(info);
    }
    trades
}

pub fn list_trades3(order_id: &str) -> Vec<TradeInfo> {
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
    from chemix_trades \
    where  (taker_order_id='{}' or maker_order_id='{}')",
        order_id,order_id
    );
    let mut trades: Vec<TradeInfo> = Vec::new();
    info!("list_trades_sql {}", sql);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        let side_str: String = row.get(10);
        let side = order::Side::from(side_str.as_str());
        let info = TradeInfo {
            id: row.get(0),
            block_height: row.get(1), //todo: 待加逻辑
            transaction_hash: row.get(2),
            hash_data: row.get(3),
            status: TradeStatus::from(row.get::<usize, &str>(4usize)), //row.get(3),
            market_id: row.get(5),
            taker: row.get(6),
            maker: row.get(7),
            price: U256::from_str_radix(row.get::<usize, &str>(8), 10).unwrap(),
            amount: U256::from_str_radix(row.get::<usize, &str>(9), 10).unwrap(),
            taker_side: side,
            maker_order_id: row.get(11),
            taker_order_id: row.get(12),
            updated_at: row.get(13),
            created_at: row.get(14),
        };
        trades.push(info);
    }
    trades
}

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

pub fn update_trade_by_hash(status: TradeStatus, hash_data: &str,block_height: u32) {
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
    let mut sum = U256::from(0);
    let rows = crate::query(sql.as_str()).unwrap();
    for row in rows {
        sum += U256::from_str_radix(row.get::<usize, &str>(0), 10).unwrap()
    }
    sum
}
