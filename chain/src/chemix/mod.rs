/// chemix main contract
pub mod chemix_main;
/// chemix storage contract
pub mod storage;
/// chemix vault contract
pub mod vault;

use crate::k256::ecdsa::SigningKey;

use ethers::prelude::*;
use std::marker::PhantomData;
use std::sync::Arc;

/// chemix contract client for call
#[derive(Clone)]
pub struct ChemixContractClient<C> {
    client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    contract_addr: H160,
    phantom: PhantomData<C>,
}
