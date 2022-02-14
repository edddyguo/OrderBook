use std::borrow::BorrowMut;
use std::collections::HashMap;
use rustc_serialize::json;
use serde::Serialize;
use std::ops::{Deref, Index, Sub};
use crate::{AddBook, LastTrade, EngineBook, AddBook2, LastTrade2};
//use ethers::{prelude::*,types::{U256}};
use serde::Deserialize;
use utils::algorithm::sha256;
use utils::math::narrow;
use chrono::offset::LocalResult;
use chrono::offset::Local;
use std::sync::MutexGuard;
use std::time;

#[derive(RustcEncodable,Deserialize, Debug,PartialEq,Clone,Serialize)]
pub enum Status {
    #[serde(rename = "matched")]
    Matched,
    #[serde(rename = "launched")]
    Launched,
    #[serde(rename = "confirmed")] // 有效区块确认防分叉回滚
    Confirmed,
}

pub fn flush(){
    todo!()
}