pub mod chemix_main;
pub mod storage;
pub mod vault;

use crate::k256::ecdsa::SigningKey;
use anyhow::Result;
use chemix_models::order::BookOrder;
use chrono::Local;
use common::env;
use common::env::CONF;
use common::types::order::Side;
use common::types::*;
use common::utils::algorithm::{sha256, u8_arr_to_str};
use common::utils::math::MathOperation;
use ethers::prelude::*;
use ethers::types::Address;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::marker::PhantomData;
use std::ops::Mul;
use std::str::FromStr;
use std::sync::Arc;


pub enum ContratType {
    Main,
    Storage,
    Vault
}

#[derive(Clone)]
pub struct ChemixContractClient<C> {
    client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    contract_addr: H160,
    phantom: PhantomData<C>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThawBalances2 {
    pub token: Address,
    pub from: Address,
    pub amount: f64,
}

