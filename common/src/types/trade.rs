extern crate rustc_serialize;
//#[derive(Serialize)]
use crate::types::order;
use serde::{Deserialize, Serialize};


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
            Status::Matched => "matched",
            Status::Launched => "launched",
            Status::Confirmed => "confirmed",
            Status::Abandoned => "abandoned",
        }
    }
}

impl From<&str> for Status {
    fn from(status_str: &str) -> Self {
        match status_str {
            "matched" => Self::Matched,
            "launched" => Self::Launched,
            "confirmed" => Self::Confirmed,
            "abandoned" => Self::Abandoned,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Serialize, Debug,Deserialize)]
pub struct AggTrade {
    pub id: String,
    pub price: f64,
    pub amount: f64,
    pub height: i32,
    pub taker_side: order::Side,
}
