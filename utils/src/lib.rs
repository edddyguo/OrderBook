extern crate rust_decimal;

pub mod algorithm;
pub mod math;
pub mod time;
pub mod env;

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
