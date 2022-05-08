extern crate rustc_serialize;

use ethers_core::types::Address;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum Status {
    #[serde(rename = "pending")]
    Pending,
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
            Status::Pending => "pending",
            Status::Launched => "launched",
            Status::Confirmed => "confirmed",
            Status::Abandoned => "abandoned",
        }
    }
}

impl From<&str> for Status {
    fn from(status_str: &str) -> Self {
        match status_str {
            "pending" => Self::Pending,
            "launched" => Self::Launched,
            "confirmed" => Self::Confirmed,
            "abandoned" => Self::Abandoned,
            _ => unreachable!(),
        }
    }
}

///user's thaw balances
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThawBalancesVO {
    /// token erc20 address
    pub token: Address,
    /// user's address
    pub from: Address,
    /// amount of balances
    pub amount: f64,
}
