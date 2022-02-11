use rust_decimal::Decimal;
use std::any::Any;
use std::ffi::CString;
use std::fmt::Debug;
use rust_decimal::prelude::ToPrimitive;
use std::ops::Deref;

pub trait MathOperation {
    fn to_fix(&self, precision: u32) -> f64;
    fn to_nano(&self) -> u64;
}

//fixme: 再次检查丢精度问题
impl MathOperation for f64 {
    fn to_fix(&self, precision: u32) -> f64 {
        println!("to_fix_self---{}",self);

        let times = 10_u32.pow(precision);
        let number_tmp = self * times as f64;
        let real_number = number_tmp.round();
        let decimal_number = Decimal::new(real_number as i64, precision);
        let scaled = decimal_number.to_f64().unwrap();
        scaled
    }

    fn to_nano(&self) -> u64 {
        println!("self---{}",self);
        let test3 = self.deref().to_fix(8);
        println!("self2---{}",test3);
        let test1 = (*self * 100_000_000.00f64);
        println!("test1---{}",test1);
        test1.to_fix(8) as u64

    }
}