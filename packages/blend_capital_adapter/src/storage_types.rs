use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub struct AssetEpochPrincipal {
    pub epoch: u64,        // Current epoch number
    pub principal: i128,   // Principal at start of epoch (includes previous yields)
    pub withdrawals: i128, // Total withdrawals during this epoch
    pub last_updated: u64, // Timestamp of last update
}