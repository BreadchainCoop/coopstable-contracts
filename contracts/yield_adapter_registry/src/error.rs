use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
/// Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
/// YieldAdapterRegistry error reporting. YieldAdapterRegistry specific errors start at 400
pub enum YieldAdapterRegistryError {
    InternalError = 1,
    AlreadyInitializedError = 3,
    UnauthorizedError = 4,
    NegativeAmountError = 8,
    BalanceError = 10,
    OverflowError = 12,

    // YieldAdapterRegistryError Errors
    InvalidYieldAdapter = 1100,
}
