use postgres::{config::Config, error::Error, row::SimpleQueryRow, Client, NoTls};
use slog::{debug, error, info};
use utils::algorithm::sha256;
use utils::time::get_current_time;
use crate::order::Side;
use serde::Serialize;


extern crate rustc_serialize;

#[derive(Serialize,Debug)]
pub struct TradeInfo {
    pub id: String,
    pub transaction_id: u32,
    pub transaction_hash: String,
    pub status: String,
    pub market_id: String,
    pub maker: String,
    pub taker: String,
    pub price: f64,
    pub amount: f64,
    pub taker_side: Side,
    pub maker_order_id: String,
    pub taker_order_id: String,
    pub updated_at: String,
    pub created_at: String,
}

impl TradeInfo {
    //todo：side和status都改enum
    pub fn new(taker: String, maker: String, price: f64, amount: f64, taker_side: Side, maker_order_id: String, taker_order_id: String) -> TradeInfo {
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

pub fn insert_trades(trades: &mut Vec<Vec<String>>) {
    //之前用表名区别开发环境改为用不同数据库
    insert_trade(trades);
    insert_trade(trades);
}

pub fn insert_trade(trades: &mut Vec<Vec<String>>) {
    let mut query = format!("insert into chemix_trades values(");
    let mut tradesArr: Vec<&str> = Default::default();
    let mut index = 0;
    let trades_len = trades.len();
    // fixme:注入的写法暂时有问题，先直接拼接
    for trade in trades {
        let mut temp_value = "".to_string();
        for i in 0..trade.len() {
            if i < trade.len() - 1 {
                temp_value = format!("{}{},", temp_value, trade[i]);
            } else {
                temp_value = format!("{}{}", temp_value, trade[i]);
            }
        }
        if (index < trades_len - 1) {
            query = format!("{}{}),(", query, temp_value);
        } else {
            query = format!("{}{})", query, temp_value);
        }
        let mut str_trade: Vec<&str> = Default::default();
        for item in trade {
            str_trade.push(&*item);
        }
        tradesArr.append(str_trade.as_mut());
        index += 1;
    }
    let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
    // let mut result = crate::CLIENTDB.lock().unwrap().execute(&*query, &tradesArr[0..tradesArr.len()]);
    if let Err(err) = result {
        //error!("insert trade sql={} failed {:?}", query, err);
        if !crate::restartDB() {
            return;
        }
        //&[&bar, &baz],
        result = crate::CLIENTDB.lock().unwrap().execute(&*query, &[]);
    }
    let rows = result.unwrap();
    //info!("insert trade successful insert {:?} rows,sql={}",rows, query);
}