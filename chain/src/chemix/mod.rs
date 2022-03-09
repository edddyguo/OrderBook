pub mod chemix_main;
pub mod storage;
pub mod vault;

use crate::k256::ecdsa::SigningKey;

use ethers::prelude::*;
use ethers::types::Address;
use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

use std::sync::Arc;

pub enum ContratType {
    Main,
    Storage,
    Vault,
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
