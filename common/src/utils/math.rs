use rust_decimal::Decimal;

use rust_decimal::prelude::ToPrimitive;
use std::ops::Div;

use ethers_core::types::U256;

use num::pow::Pow;

const U256_ZERO : U256 = U256([0;4]);

#[macro_export]
macro_rules! teen_power{
    ($a:expr)=>{
        {
            U256::from(10u32).pow(U256::from($a))
        }
    }
}

pub trait MathOperation {
    fn to_fix(&self, precision: u32) -> f64;
    fn to_nano(&self) -> u64;
}

//fixme: 再次检查丢精度问题
impl MathOperation for f64 {
    fn to_fix(&self, precision: u32) -> f64 {
        let times = 10_u32.pow(precision);
        let number_tmp = self * times as f64;
        let real_number = number_tmp.round();
        let decimal_number = Decimal::new(real_number as i64, precision);
        let scaled = decimal_number.to_f64().unwrap();
        scaled
    }

    //fixme： 失效了
    fn to_nano(&self) -> u64 {
        let test1 = *self * 100_000_000.00f64;
        //test1.to_fix(8) as u64
        test1.floor() as u64
    }
}

/***
pub trait ExtensionNat {
    fn to_int<T>(&self) -> T
        where
            T: FromStr,
            T::Err: std::fmt::Debug;
}
impl ExtensionNat for Nat {
    fn to_int<T>(&self) -> T
        where
            T: FromStr,
            T::Err: std::fmt::Debug,
    {
        self.to_string().replace("_", "").parse::<T>().unwrap()
    }
}
            fee: self.fee.to_int::<u64>(),

 */

pub fn narrow(ori: u64) -> f64 {
    let decimal_number = Decimal::new(ori as i64, 8);
    decimal_number.to_f64().unwrap()
}

//fixme:考虑用其他库,硬编码精度为8位，decimal超过37的话仍溢出，目前业务不会触发
//fixme: f64的有效精度为16位,当前业务做一定的取舍，总账对上就行
pub fn u256_to_f64(ori: U256, decimal: u32) -> f64 {
    let decimal_value = U256::from(10u32).pow(U256::from(decimal - 8));
    let dist_int = ori.div(decimal_value);
    let mut dist = Decimal::from(dist_int.as_u128());
    dist.set_scale(8);
    dist.to_f64().unwrap()
}

#[cfg(test)]
mod tests {
    use ethers_core::types::U256;

    use crate::utils::math::u256_to_f64;

    #[test]
    fn test_u256_to_f64() {
        //let a = U256::from_str_radix("123456789012345178901234567890012345678901234567890",10).unwrap();
        let a = U256::from_str_radix("1234567890123451789012345678912", 10).unwrap();
        let res1 = u256_to_f64(a, 22);
        assert_eq!(res1, 123456789.01234517);
        let a = U256::from_str_radix("1", 10).unwrap();
        let res2 = u256_to_f64(a, 22);
        assert_eq!(res2, 0.0);
    }
}
