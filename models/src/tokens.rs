extern crate rustc_serialize;

use ethers_core::types::U256;
//#[derive(Serialize)]
use serde::Serialize;
use crate::struct2array;

/***
create table chemix_tokens(
 symbol text primary key,
 name text,
 address text ,
 front_decimals integer,
 base_contract_decimal integer,
 cvt_url text,
 show_cvt boolean,
);
*/
#[derive(Serialize, Debug, Default)]
pub struct Token {
    pub symbol: String,
    pub name: String,
    pub address: String,
    pub front_decimals: i32,
    pub base_contract_decimal: i32,
    pub cvt_url: String,
    pub show_cvt: bool,
}


pub fn list_tokens() -> Vec<Token> {

    let sql = format!("select symbol,name,address,front_decimals,\
    base_contract_decimal,cvt_url,show_cvt from chemix_tokens ");
    let rows = crate::query(sql.as_str()).unwrap();
    let mut tokens = Vec::new();
    info!("get_snapshot: raw sql {}", sql);
    for row in rows {
        tokens.push(Token {
            symbol: row.get(0),
            name: row.get(1),
            address: row.get(2),
            front_decimals: row.get(3),
            base_contract_decimal: row.get(4),
            cvt_url: row.get(5),
            show_cvt: row.get(6),
        });
    }
    tokens
}