extern crate rust_decimal;
///The encryption algorithm
pub mod algorithm;
///Mathematical calculations
pub mod math;
/// time
pub mod time;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
