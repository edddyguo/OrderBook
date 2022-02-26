extern crate rustc_serialize;

use std::str::FromStr;
use ethers_core::types::U256;
use jsonrpc_http_server::tokio::prelude::future::Ok;
use serde::Deserialize;

//#[derive(Serialize)]
use serde::Serialize;
use std::fmt::Display;
use crate::types::order::Side::{Buy, Sell};

#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum Status {
    #[serde(rename = "full_filled")]
    FullFilled,
    #[serde(rename = "partial_filled")]
    PartialFilled,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "canceled")]
    Canceled,
    #[serde(rename = "abandoned")]
    Abandoned,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullFilled => "full_filled",
            Self::Abandoned => "abandoned",
            Self::PartialFilled => "partial_filled",
            Self::Pending => "pending",
            Self::Canceled => "canceled",
        }
    }
}

impl From<&str> for Status {
    fn from(status_str: &str) -> Self {
        match status_str {
            "full_filled" => Self::FullFilled,
            "partial_filled" => Self::PartialFilled,
            "pending" => Self::Pending,
            "canceled" => Self::Canceled,
            "abandoned" => Self::Abandoned,
            _ => unreachable!()
        }
    }
}

#[derive(RustcEncodable, Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum Side {
    #[serde(rename = "buy")]
    Buy,
    #[serde(rename = "sell")]
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Buy => "buy",
            Sell => "sell",
        }
    }

    pub fn contrary(&self) -> Side {
        match self {
            Buy => Sell,
            Sell => Buy,
        }
    }
}

impl From<&str> for Side {
    fn from(side_str: &str) -> Self {
        match side_str {
            "buy" => Self::Buy,
            "sell" => Self::Sell,
            _ => unreachable!()
        }
    }
}