#[derive(Clone, Copy, Debug, PartialEq, EnumIter, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    MarketIdNotExist = 101,
    AddressIllegal = 201,
    UnknownError = 301,
}