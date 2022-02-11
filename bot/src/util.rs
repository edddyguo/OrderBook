//use chrono::offset::LocalResult;
//use chrono::prelude::*;
use num::ToPrimitive;
//use ring::digest;
use rust_decimal::Decimal;
use std::any::Any;
use std::ffi::CString;
use std::fmt::Debug;

// use crate::consume::engine::EngineTrade;

pub trait MathOperation {
    fn to_fix(&self, precision: u32) -> f64;
}

impl MathOperation for f64 {
    fn to_fix(&self, precision: u32) -> f64 {
        let times = 10_u32.pow(precision);
        let number_tmp = self * times as f64;
        let real_number = number_tmp.round();
        let decimal_number = Decimal::new(real_number as i64, precision);
        let scaled = decimal_number.to_f64().unwrap();
        scaled
    }
}