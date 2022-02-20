extern crate rustc_serialize;

//#[derive(Serialize)]
use serde::Serialize;

#[derive(Serialize, Debug, Default)]
pub struct MarketInfo {
    pub id: String,
    base_token_address: String,
    base_token_symbol: String,
    base_contract_decimal: u32,
    base_front_decimal: u32,
    quote_token_address: String,
    quote_token_symbol: String,
    quote_contract_decimal: u32,
    quote_front_decimal: u32,
}

pub fn list_markets() -> Vec<MarketInfo> {
    let sql = "select id,base_token_address,base_token_symbol,base_contract_decimal,\
    base_front_decimal,quote_token_address,quote_token_symbol,quote_contract_decimal,\
    quote_front_decimal from chemix_markets where online=true";

    let mut markets: Vec<MarketInfo> = Vec::new();
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    if let Err(err) = result {
        println!("get_active_address_num failed {:?}", err);
        if !crate::restartDB() {
            return markets;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    }
    let rows = result.unwrap();
    for row in rows {
        let info = MarketInfo {
            id: row.get(0),
            base_token_address: row.get(1),
            base_token_symbol: row.get(2),
            base_contract_decimal: row.get(3),
            base_front_decimal: row.get(4),
            quote_token_address: row.get(5),
            quote_token_symbol: row.get(6),
            quote_contract_decimal: row.get(7),
            quote_front_decimal: row.get(8),
        };
        markets.push(info);
    }
    markets
}

//user num from scope time age to now or no time limit
pub fn get_user_number(scope: Option<u64>) -> u32 {
    let sql = "select count(1) from (select account from chemix_orders group by account) as users";
    let mut user_num = 0u32;
    let mut result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    if let Err(err) = result {
        println!("get_active_address_num failed {:?}", err);
        if !crate::restartDB() {
            return user_num;
        }
        result = crate::CLIENTDB.lock().unwrap().query(sql, &[]);
    }
    let rows = result.unwrap();
    user_num  = rows[0].get(0);
    user_num
}
