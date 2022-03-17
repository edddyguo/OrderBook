extern crate rustc_serialize;

use std::collections::HashMap;
use ethers_core::types::{I256, U256};
use serde::{Deserialize, Serialize};

//#[derive(Serialize)]

#[derive(Clone, Serialize,Deserialize,Default,PartialEq)]
pub struct Depth {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

#[derive(Clone, Serialize, Debug)]
pub struct RawDepth {
    pub asks: HashMap<U256, I256>,
    pub bids: HashMap<U256, I256>,
}