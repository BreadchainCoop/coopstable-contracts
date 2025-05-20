use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]

/// Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
/// AccessControl error reporting. CUSDManager specific errors start at 100
pub enum LendingYieldControllerError {
    InternalError = 1,
    AlreadyInitializedError = 3,
    UnauthorizedError = 4,
    NegativeAmountError = 8,
    BalanceError = 10,
    OverflowError = 12,

    // AccessControl Errors
    MemberAlreadyExists = 200,
    UnAuhtorizedRole = 201,
}
