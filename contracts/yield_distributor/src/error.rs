use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
/// Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
/// YieldDistributorError error reporting. YieldDistributor specific errors start at 400
pub enum YieldDistributorError {
    InternalError = 1,
    AlreadyInitializedError = 3,
    UnauthorizedError = 4,
    NegativeAmountError = 8,
    BalanceError = 10,
    OverflowError = 12,

    // YieldDistributorError Errors
    MemberAlreadyExists = 1200,
    MemberDoesNotExist = 1201,
}
