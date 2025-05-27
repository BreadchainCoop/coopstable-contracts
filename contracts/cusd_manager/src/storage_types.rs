use soroban_sdk::{contracttype, symbol_short, Symbol};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
pub(crate) const CUSD_ADMIN: Symbol = symbol_short!("CUSD_ADMN");
pub(crate) const CUSD_ADDRESS_KEY: Symbol = symbol_short!("cUSD");
pub(crate) const YIELD_CONTROLLER: Symbol = symbol_short!("YLDWCTR");

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Manager,
}
