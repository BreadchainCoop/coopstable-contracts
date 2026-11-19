use soroban_sdk::{contracterror, contracttype, Address};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
    /// Deposit of an asset this adapter is not bound to
    UnsupportedAsset = 1300,
}

#[derive(Clone)]
#[repr(u32)]
#[contracttype]
pub enum RequestType {
    Supply = 0,
    Withdraw = 1,
    SupplyCollateral = 2,
    WithdrawCollateral = 3,
    Borrow = 4,
    Repay = 5,
    FillUserLiquidationAuction = 6,
    FillBadDebtAuction = 7,
    FillInterestAuction = 8,
    DeleteLiquidationAuction = 9,
}

#[derive(Clone)]
#[contracttype]
pub struct Request {
    pub request_type: u32,
    pub address: Address,
    pub amount: i128,
}
