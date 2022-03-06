pub mod env;
pub mod types;
pub mod utils;

#[macro_use]
extern crate lazy_static;
/****
todo: hashmap的update
use std::collections::HashMap;
use ethers_core::k256::U256;

pub trait Crud {
    fn update_U256<k>(&mut self,key:k,value: U256) -> bool;
}

impl Crud for HashMap<K,U256> {
    fn update_U256<K>(&mut self,key: K,value: U256) -> bool {
        let stat = self.entry(key).or_insert(value.clone());
        *stat += value;
        true
    }
}

 */

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
