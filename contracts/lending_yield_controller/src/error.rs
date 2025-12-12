use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
/// Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
/// LendingYieldControllerError error reporting. CUSDManager specific errors start at 100
pub enum LendingYieldControllerError {
    InternalError = 1,
    AlreadyInitializedError = 3,
    UnauthorizedError = 4,
    NegativeAmountError = 8,
    BalanceError = 10,
    OverflowError = 12,

    // LendingYieldControllerError Errors
    UnsupportedAsset = 1000,
    YieldUnavailable = 1001,

    // Multi-stage harvest errors
    /// No pending harvest exists for this protocol/asset
    NoPendingHarvest = 1002,
    /// A harvest is already in progress for this protocol/asset
    HarvestAlreadyInProgress = 1003,
    /// Invalid harvest state for this operation
    InvalidHarvestState = 1004,
    /// No yield available to harvest
    NoYieldToHarvest = 1005,
}
