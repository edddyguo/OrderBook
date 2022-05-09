//! some usually type or function
#![deny(missing_docs)]
#![deny(warnings)]
//#![deny(unused_crate_dependencies)]
//#![warn(perf)]

///env config
pub mod env;
///redis message queue
pub mod queue;
///chemix types
pub mod types;
///Some general utils
pub mod utils;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
