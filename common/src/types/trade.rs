extern crate rustc_serialize;

use serde::Deserialize;

//#[derive(Serialize)]
use serde::Serialize;

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
