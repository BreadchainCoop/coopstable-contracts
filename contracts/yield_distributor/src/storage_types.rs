use soroban_sdk::{contracttype, symbol_short, Address, Symbol, Vec};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const PERSISTENT_BUMP_AMOUNT: u32 = 90 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;
pub(crate) const PERSISTENT_LIFETIME_THRESHOLD: u32 = PERSISTENT_BUMP_AMOUNT - DAY_IN_LEDGERS;

// Keys for instance storage
pub (crate) const CURRENT_EPOCH_KEY: Symbol = symbol_short!("EPOCH");

// Structure to hold distribution configuration
#[derive(Clone)]
#[contracttype]
pub struct DistributionConfig {
    pub treasury_share_bps: u32,  // Basis points (e.g., 1000 = 10%)
    pub distribution_period: u64, // In seconds
}

// Structure for storing member data
#[derive(Clone)]
#[contracttype]
pub struct Member {
    pub address: Address,
    pub active: bool,
    pub joined_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct Distribution {
    pub distribution_end_timestamp: u64,
    pub distribution_start_timestamp: u64,
    pub distribution_total: i128,
    pub distribution_treasury: i128,
    pub distribution_member: i128,
    pub member_count: u32,
    pub is_processed: bool, // In seconds
    pub epoch: u64,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Owner,
    YieldController,
    Treasury,
    Member(Address),   // Map address to Member
    Members,           // Vec of all member addresses
    Distributions,     // Vec of all distribution timestamps
    Distribution(u64), // distribution to epoch
    DistributionConfig,
    Epoch(u64),
    EpochStartTimestamp(u64),
    TotalDistributed,
    EpochMembers(u64), // Store member list for specific epoch
    ActiveMemberCount, // Cache active member count
}
