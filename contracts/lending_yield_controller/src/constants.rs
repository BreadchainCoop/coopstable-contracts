use::soroban_sdk::{Symbol, symbol_short};
use crate::yield_adapter_registry::SupportedYieldType; 

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub (crate) const ADAPTER_REGISTRY_KEY: Symbol = symbol_short!("AR");
pub (crate) const CUSD_MANAGER_KEY: Symbol = symbol_short!("CM");
pub (crate) const YIELD_TYPE: SupportedYieldType = SupportedYieldType::Lending;
pub (crate) const YIELD_DISTRIBUTOR_KEY: Symbol = symbol_short!("YD");
