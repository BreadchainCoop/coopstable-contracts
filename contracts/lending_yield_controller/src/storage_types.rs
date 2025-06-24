use ::soroban_sdk::contracttype;
use yield_adapter::contract_types::SupportedYieldType;

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
pub(crate) const YIELD_TYPE: SupportedYieldType = SupportedYieldType::Lending;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Owner,
    Admin,
    CUSDManager,
    AdapterRegistry,
    YieldDistributor,
}