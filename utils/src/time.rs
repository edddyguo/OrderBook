use chrono::offset::LocalResult;
use chrono::prelude::*;
use num::ToPrimitive;
use rust_decimal::Decimal;
use std::any::Any;
use std::ffi::CString;
use std::fmt::Debug;

pub fn get_current_time() -> String {
    let dt: DateTime<Local> = Local::now();
    dt.format("%Y-%m-%d %H:%M:%S.%f").to_string()
}