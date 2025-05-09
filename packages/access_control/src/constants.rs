use soroban_sdk::{symbol_short, Symbol};

// Constants for storage lifetimes - configurable through initialization
pub const DAY_IN_LEDGERS: u32 = 17280;
pub const DEFAULT_INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub const DEFAULT_INSTANCE_LIFETIME_THRESHOLD: u32 = DEFAULT_INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

// Default admin role - similar to OpenZeppelin's DEFAULT_ADMIN_ROLE
pub const DEFAULT_ADMIN_ROLE: Symbol = symbol_short!("ADMIN");

/// Storage key for roles mapping
pub const ROLES_KEY: Symbol = symbol_short!("ROLES");
