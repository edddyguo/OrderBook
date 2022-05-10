use serde::{Deserialize,Serialize};

#[derive(Deserialize, Debug, PartialEq, Clone, Serialize)]
pub enum ChainStatus {
    #[serde(rename = "Stoped")]
    Stoped,
    #[serde(rename = "Forked")]
    Forked,
    #[serde(rename = "Healthy")]
    Healthy,
}

impl ChainStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stoped => "Stoped",
            Self::Forked => "Forked",
            Self::Healthy => "Healthy",
        }
    }
}

impl From<&str> for ChainStatus {
    fn from(status_str: &str) -> Self {
        match status_str {
            "Stoped" => Self::Stoped,
            "Forked" => Self::Forked,
            "Healthy" => Self::Healthy,
            _ => unreachable!(),
        }
    }
}

