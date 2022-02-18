use std::str::FromStr;
use ethers_core::types::U256;
use crate::order::Side;
use crate::struct2array;
use crate::Side::{Buy, Sell};
use chemix_utils::algorithm::sha256;
use chemix_utils::time::get_current_time;
use serde::{Serialize,Deserialize};

extern crate rustc_serialize;


#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum Status {
    #[serde(rename = "matched")]
    Matched,
    #[serde(rename = "launched")]
    Launched,
    #[serde(rename = "confirmed")] // 有效区块确认防分叉回滚
    Confirmed,
    #[serde(rename = "abandoned")] // which is abandoned because of chain forked
    Abandoned,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Matched => "matched",
            Launched => "launched",
            Confirmed => "confirmed",
            Abandoned => "abandoned",
        }
    }
}

#[derive(Serialize, Debug)]
pub struct TradeInfo {
    pub id: String,
    pub transaction_id: i32,
    pub transaction_hash: String,
    pub status: String,
    pub market_id: String,
    pub maker: String,
    pub taker: String,
    pub price: U256,
    pub amount: U256,
    pub taker_side: Side,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub updated_at: String,
    pub created_at: String,
}

impl TradeInfo {
    //todo：side和status都改enum
    pub fn new(
        taker: String,
        maker: String,
        price: U256,
        amount: U256,
        taker_side: Side,
        maker_order_id: String,
        taker_order_id: String,
    ) -> TradeInfo {
        let now = get_current_time();
        let mut trade = TradeInfo {
            id: "".to_string(),
            transaction_id: 0, //todo: 待加逻辑
            transaction_hash: "".to_string(),
            status: "matched".to_string(),
            market_id: "BTC-USDT".to_string(),
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
    if trades.is_empty() {
        return;
    }
    let mut query = format!("insert into chemix_trades values(");
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
            query = format!("{}{}),(", query, temp_value);
        } else {
            query = format!("{}{})", query, temp_value);
        }
        let mut str_trade: Vec<String> = Default::default();
        for item in trade {
            str_trade.push(item);
        }
        tradesArr2.append(&mut str_trade);
        index += 1;
    }
    println!("----query==={}", query);
    let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
    // let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &tradesArr[0..tradesArr.len()]);
    if let Err(_err) = result {
        //error!("insert trade sql={} failed {:?}", query, err);
        if !crate::restartDB() {
            return;
        }
        //&[&bar, &baz],
        result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
    }
    let _rows = result.unwrap();
    //info!("insert trade successful insert {:?} rows,sql={}",rows, query);
}

pub fn list_trades(num: u32) -> Vec<TradeInfo> {
    let sql = format!(
        "select \
    id,\
    transaction_id,\
    transaction_hash,\
    status,\
    market_id,\
    maker,\
    taker,\
    price,\
    amount,\
    taker_side,\
    maker_order_id, \
    taker_order_id,\
    cast(created_at as text), \
    cast(updated_at as text) \
    from chemix_trades \
    where market_id='BTC-USDT' order by created_at ASC limit {}",
        num
    );
    let mut trades: Vec<TradeInfo> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    if let Err(_err) = result {
        //info!("list_available_orders failed {:?}", err);
        if !crate::restartDB() {
            return trades;
        }
        result = crate::CLIENTDB.lock().unwrap().query(&*sql, &[]);
    }
    let rows = result.unwrap();
    for row in rows {
        let test1: String = row.get(9);
        let side = match test1.as_str() {
            "sell" => Sell,
            "buy" => Buy,
            _ => {
                println!("side {}", test1.as_str());
                assert!(false);
                Buy
            }
        };
        let info = TradeInfo {
            id: row.get(0),
            transaction_id: row.get(1), //todo: 待加逻辑
            transaction_hash: row.get(2),
            status: row.get(3),
            market_id: row.get(4),
            taker: row.get(5),
            maker: row.get(6),
            price: U256::from_str(row.get::<usize,&str>(7)).unwrap(),
            amount: U256::from_str(row.get::<usize,&str>(8)).unwrap(),
            taker_side: side,
            maker_order_id: row.get(10),
            taker_order_id: row.get(11),
            updated_at: row.get(12),
            created_at: row.get(13),
        };
        trades.push(info);
    }
    trades
}
