#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorCode {
    MarketIdNotExist = 101,
    AddressIllegal = 201,
    UnknownError = 301,
}
