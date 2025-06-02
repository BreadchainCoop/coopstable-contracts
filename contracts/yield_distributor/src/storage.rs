use crate::storage_types::{
    DataKey, Distribution, DistributionConfig, Member, DISTRIBUTION_PERIOD_KEY,
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, LAST_DISTRIBUTION_KEY, TREASURY_KEY,
    TREASURY_SHARE_KEY, YIELD_CONTROLLER_KEY,
};
use soroban_sdk::{Address, Env, Map, Symbol, Vec};

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn get_treasury(e: &Env) -> Address {
    extend_instance(e);
    e.storage().instance().get(&TREASURY_KEY).unwrap()
}

pub fn set_treasury(e: &Env, treasury: &Address) {
    extend_instance(e);
    e.storage().instance().set(&TREASURY_KEY, treasury);
}

pub fn get_treasury_share_bps(e: &Env) -> u32 {
    extend_instance(e);
    e.storage().instance().get(&TREASURY_SHARE_KEY).unwrap()
}

pub fn set_treasury_share_bps(e: &Env, share_bps: u32) {
    if share_bps > 10000 {
        panic!("Treasury share cannot exceed 100% (10000 basis points)");
    }
    extend_instance(e);
    e.storage().instance().set(&TREASURY_SHARE_KEY, &share_bps);
}

// Yield controller getter/setter
pub fn get_yield_controller(e: &Env) -> Address {
    extend_instance(e);
    e.storage().instance().get(&YIELD_CONTROLLER_KEY).unwrap()
}

pub fn set_yield_controller(e: &Env, controller: &Address) {
    extend_instance(e);
    e.storage()
        .instance()
        .set(&YIELD_CONTROLLER_KEY, controller);
}

pub fn get_distribution_period(e: &Env) -> u64 {
    extend_instance(e);
    e.storage()
        .instance()
        .get(&DISTRIBUTION_PERIOD_KEY)
        .unwrap()
}

pub fn set_distribution_period(e: &Env, period: u64) {
    extend_instance(e);
    e.storage()
        .instance()
        .set(&DISTRIBUTION_PERIOD_KEY, &period);
}

pub fn get_last_distribution(e: &Env) -> u64 {
    extend_instance(e);
    if let Some(timestamp) = e.storage().instance().get(&LAST_DISTRIBUTION_KEY) {
        timestamp
    } else {
        0
    }
}

pub fn set_last_distribution(e: &Env, timestamp: u64) {
    extend_instance(e);
    e.storage()
        .instance()
        .set(&LAST_DISTRIBUTION_KEY, &timestamp);
}

// Member management
pub fn add_member(e: &Env, address: &Address) {
    extend_instance(e);

    // Create member record
    let member = Member {
        address: address.clone(),
        active: true,
        joined_at: e.ledger().timestamp(),
    };

    // Save in persistent storage
    let key = DataKey::Member(address.clone());

    e.storage().persistent().set(&key, &member);
    e.storage()
        .persistent()
        .extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    // Add to members list if not already there
    let members_key = DataKey::Members;
    let mut members: Vec<Address> = match e.storage().persistent().get(&members_key) {
        Some(existing) => existing,
        None => Vec::new(e),
    };

    if !members.iter().any(|a| a == address.clone()) {
        members.push_back(address.clone());
        e.storage().persistent().set(&members_key, &members);
        e.storage().persistent().extend_ttl(
            &members_key,
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
    }
}

pub fn remove_member(e: &Env, address: &Address) {
    extend_instance(e);

    // Update member record to inactive
    let key = DataKey::Member(address.clone());
    if let Some(mut member) = e.storage().persistent().get::<DataKey, Member>(&key) {
        member.active = false;
        e.storage().persistent().set(&key, &member);
        e.storage().persistent().extend_ttl(
            &key,
            INSTANCE_LIFETIME_THRESHOLD,
            INSTANCE_BUMP_AMOUNT,
        );
    }

    // Note: We don't remove from the members list to maintain history
}

pub fn get_member(e: &Env, address: &Address) -> Option<Member> {
    extend_instance(e);
    let key = DataKey::Member(address.clone());
    e.storage().persistent().get(&key)
}

pub fn get_active_members(e: &Env) -> Vec<Address> {
    extend_instance(e);

    let members_key = DataKey::Members;

    if let Some(all_members) = e
        .storage()
        .persistent()
        .get::<DataKey, Vec<Address>>(&members_key)
    {
        let mut active = Vec::new(e);

        for address in all_members.iter() {
            if let Some(member) = get_member(e, &address) {
                if member.active {
                    active.push_back(address);
                }
            }
        }

        return active;
    }

    Vec::new(e)
}

pub fn count_active_members(e: &Env) -> u32 {
    get_active_members(e).len()
}

// Distribution management
pub fn record_distribution(e: &Env, total: i128, treasury_amount: i128, member_amount: i128) {
    extend_instance(e);

    let timestamp = e.ledger().timestamp();

    let distribution = Distribution {
        timestamp,
        total_amount: total,
        treasury_amount,
        member_amount,
        member_count: count_active_members(e),
    };

    // Save distribution record
    let key = DataKey::Distribution(timestamp);
    e.storage().persistent().set(&key, &distribution);
    e.storage()
        .persistent()
        .extend_ttl(&key, INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    // Add to distributions list
    let distributions_key = DataKey::Distributions;
    let mut distributions: Vec<u64> = match e.storage().persistent().get(&distributions_key) {
        Some(existing) => existing,
        None => Vec::new(e),
    };

    distributions.push_back(timestamp);
    e.storage()
        .persistent()
        .set(&distributions_key, &distributions);
    e.storage().persistent().extend_ttl(
        &distributions_key,
        INSTANCE_LIFETIME_THRESHOLD,
        INSTANCE_BUMP_AMOUNT,
    );

    // Update last distribution timestamp
    set_last_distribution(e, timestamp);
}

pub fn get_distribution_config(e: &Env) -> DistributionConfig {
    extend_instance(e);

    DistributionConfig {
        treasury: get_treasury(e),
        treasury_share_bps: get_treasury_share_bps(e),
        distribution_period: get_distribution_period(e),
        last_distribution: get_last_distribution(e),
    }
}

pub fn read_next_distribution(e: &Env) -> u64 {
    let config = get_distribution_config(e);
    let next_time = config.last_distribution + config.distribution_period;
    next_time
}

pub fn check_distribution_availability(e: &Env) -> bool {
    let current_time = e.ledger().timestamp();
    let next_time = read_next_distribution(e);
    current_time >= next_time
}