#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AdapterError {
    InsufficientBalance = 1,
    LendingOperationFailed = 2,
    Unauthorized = 3,
}