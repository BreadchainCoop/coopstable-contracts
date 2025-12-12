use ::soroban_sdk::{contracttype, Address, Symbol};
use yield_adapter::contract_types::SupportedYieldType;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
pub(crate) const YIELD_TYPE: SupportedYieldType = SupportedYieldType::Lending;

/// State of a pending harvest operation
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
#[repr(u32)]
pub enum HarvestState {
    /// No pending harvest
    None = 0,
    /// Yield has been harvested (withdrawn from protocol)
    Harvested = 1,
    /// Yield has been recompounded (re-deposited to protocol)
    Recompounded = 2,
}

/// Pending harvest data stored between multi-stage operations
#[derive(Clone)]
#[contracttype]
pub struct PendingHarvest {
    pub protocol: Symbol,
    pub asset: Address,
    pub amount: i128,
    pub state: HarvestState,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Owner,
    Admin,
    CUSDManager,
    AdapterRegistry,
    YieldDistributor,
    /// Pending harvest for a specific protocol/asset pair
    PendingHarvest(Symbol, Address),
}