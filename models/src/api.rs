extern crate rustc_serialize;

//#[derive(Serialize)]
use serde::Serialize;

#[derive(Serialize, Debug, Default, Clone)]
pub struct MarketInfo {
    pub id: String,
    pub base_token_address: String,
    pub base_token_symbol: String,
    pub base_contract_decimal: i32,
    pub base_front_decimal: i32,
    pub quote_token_address: String,
    pub quote_token_symbol: String,
    pub quote_contract_decimal: i32,
    pub quote_front_decimal: i32,
}

pub fn list_markets() -> Vec<MarketInfo> {
    let sql = "select id,base_token_address,base_token_symbol,base_contract_decimal,\
    base_front_decimal,quote_token_address,quote_token_symbol,quote_contract_decimal,\
    quote_front_decimal from chemix_markets where online=true";

    let mut markets: Vec<MarketInfo> = Vec::new();
    let rows = crate::query(sql).unwrap();
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

pub fn get_markets(id: &str) -> MarketInfo {
    let sql = format!(
        "select id,base_token_address,base_token_symbol,base_contract_decimal,\
    base_front_decimal,quote_token_address,quote_token_symbol,quote_contract_decimal,\
    quote_front_decimal from chemix_markets where online=true and id=\'{}\'",
        id
    );
    let execute_res = crate::query(sql.as_str()).unwrap();
    info!("get_markets: raw sql {}", sql);
    //id只有一个
    MarketInfo {
        id: execute_res[0].get(0),
        base_token_address: execute_res[0].get(1),
        base_token_symbol: execute_res[0].get(2),
        base_contract_decimal: execute_res[0].get(3),
        base_front_decimal: execute_res[0].get(4),
        quote_token_address: execute_res[0].get(5),
        quote_token_symbol: execute_res[0].get(6),
        quote_contract_decimal: execute_res[0].get(7),
        quote_front_decimal: execute_res[0].get(8),
    }
}

pub fn get_markets2(id: &str) -> Option<MarketInfo> {
    let sql = format!(
        "select id,base_token_address,base_token_symbol,base_contract_decimal,\
    base_front_decimal,quote_token_address,quote_token_symbol,quote_contract_decimal,\
    quote_front_decimal from chemix_markets where online=true and id=\'{}\'",
        id
    );
    let execute_res = crate::query(sql.as_str()).unwrap();
    if execute_res.is_empty() {
        return None;
    }
    info!("get_markets: raw sql {}", sql);
    //id只有一个
    Some(MarketInfo {
        id: execute_res[0].get(0),
        base_token_address: execute_res[0].get(1),
        base_token_symbol: execute_res[0].get(2),
        base_contract_decimal: execute_res[0].get(3),
        base_front_decimal: execute_res[0].get(4),
        quote_token_address: execute_res[0].get(5),
        quote_token_symbol: execute_res[0].get(6),
        quote_contract_decimal: execute_res[0].get(7),
        quote_front_decimal: execute_res[0].get(8),
    })
}
