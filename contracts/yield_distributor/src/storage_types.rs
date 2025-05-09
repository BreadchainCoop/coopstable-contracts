use soroban_sdk::{contracttype, symbol_short, Address, Symbol};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

// Keys for instance storage
pub const TREASURY_KEY: Symbol = symbol_short!("TREASURY");
pub const TREASURY_SHARE_KEY: Symbol = symbol_short!("TR_SHARE");
pub const YIELD_CONTROLLER_KEY: Symbol = symbol_short!("YC");
pub const DISTRIBUTION_PERIOD_KEY: Symbol = symbol_short!("DIST_PRD");
pub const LAST_DISTRIBUTION_KEY: Symbol = symbol_short!("LAST_DIST");

// Structure to hold distribution configuration
#[derive(Clone)]
#[contracttype]
pub struct DistributionConfig {
    pub treasury: Address,
    pub treasury_share_bps: u32,  // Basis points (e.g., 1000 = 10%)
    pub distribution_period: u64, // In seconds
    pub last_distribution: u64,   // Timestamp of last distribution
}

// Structure for storing member data
#[derive(Clone)]
#[contracttype]
pub struct Member {
    pub address: Address,
    pub active: bool,
    pub joined_at: u64,
}

// Structure for tracking distributions
#[derive(Clone)]
#[contracttype]
pub struct Distribution {
    pub timestamp: u64,
    pub total_amount: i128,
    pub treasury_amount: i128,
    pub member_amount: i128,
    pub member_count: u32,
}

// Keys for persistent storage
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Member(Address),   // Map address to Member
    Members,           // Vec of all member addresses
    Distribution(u64), // Map timestamp to Distribution
    Distributions,     // Vec of all distribution timestamps
}
