use crate::storage_types::{ 
    DataKey, 
    Distribution, 
    DistributionConfig, 
    Member, 
    CURRENT_EPOCH_KEY,
    INSTANCE_BUMP_AMOUNT, 
    INSTANCE_LIFETIME_THRESHOLD, 
    PERSISTENT_BUMP_AMOUNT, 
    PERSISTENT_LIFETIME_THRESHOLD
};
use soroban_sdk::{Address, Env, Vec};

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn extend_persistent(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_LIFETIME_THRESHOLD, PERSISTENT_BUMP_AMOUNT);
}

pub fn read_admin(e: &Env) -> Address { read_address(e, &DataKey::Admin)}
pub fn read_owner(e: &Env) -> Address { read_address(e, &DataKey::Owner)}

fn read_address(e: &Env, key: &DataKey) -> Address {
    extend_instance(e);
    e.storage().instance().get(key).unwrap()  
}

fn write_address(e: &Env, key: &DataKey, address: &Address) {
    extend_instance(e);
    e.storage().instance().set(key, address); 
}

pub fn write_admin(e: &Env, new_admin: Address) { write_address(e, &DataKey::Admin, &new_admin);}
pub fn write_owner(e: &Env, new_owner: Address) { write_address(e, &DataKey::Owner, &new_owner);}

pub fn get_treasury(e: &Env) -> Address { read_address(e, &DataKey::Treasury) }

pub fn set_treasury(e: &Env, treasury: &Address) { write_address(e, &DataKey::Treasury, treasury); }

pub fn get_treasury_share_bps(e: &Env) -> u32 { read_distribution_config(e).treasury_share_bps }

pub fn set_treasury_share_bps(e: &Env, share_bps: u32) {
    if share_bps > 10000 {
        panic!("Treasury share cannot exceed 100% (10000 basis points)");
    }
    let mut config = match e.storage().instance().get(&DataKey::DistributionConfig) {
        Some(existing) => existing,
        None => DistributionConfig {
            treasury_share_bps: 0,
            distribution_period: 0,
        }
    };
    config.treasury_share_bps = share_bps;
    write_distribution_config(e, config);
}

pub fn get_yield_controller(e: &Env) -> Address { read_address(e, &DataKey::YieldController) }

pub fn set_yield_controller(e: &Env, controller: &Address) { write_address(e, &DataKey::YieldController, controller);}

pub fn set_distribution_period(e: &Env, period: u64) {
    let mut config = match e.storage().instance().get(&DataKey::DistributionConfig) {
        Some(existing) => existing,
        None => DistributionConfig {
            treasury_share_bps: 0,
            distribution_period: 0,
        }
    };
    config.distribution_period = period;
    write_distribution_config(e, config);
}

pub fn add_member(e: &Env, address: &Address) {
    
    extend_instance(e);
    let member = Member {
        address: address.clone(),
        active: true,
        joined_at: e.ledger().timestamp(),
    };
    
    let key = DataKey::Member(address.clone());
    e.storage().persistent().set(&key, &member);
    extend_persistent(e, &key);

    let members_key = DataKey::Members;
    let mut members: Vec<Address> = match e.storage().persistent().get(&members_key) {
        Some(existing) => existing,
        None => Vec::new(e),
    };
    if !members.iter().any(|a| a == address.clone()) {
        members.push_back(address.clone());
        e.storage().persistent().set(&members_key, &members);
        extend_persistent(e, &members_key);
    }
    
    // Update current distribution with new member list
    update_current_distribution_members(e);
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

    // Update current distribution with new member list
    update_current_distribution_members(e);
    
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

fn update_current_distribution_members(e: &Env) {
    let current_epoch = read_epoch_current(e);
    if e.storage().persistent().has(&DataKey::Distribution(current_epoch)) {
        let mut current_distribution = read_distribution(e, current_epoch);
        current_distribution.members = get_active_members(e);
        write_distribution(e, current_epoch, current_distribution);
    }
}

pub fn record_distribution(e: &Env, total: i128, treasury_amount: i128, member_amount: i128) {
    
    let epoch = read_epoch_current(e);
    
    let mut distribution: Distribution = read_distribution_of_current_epoch(e);
    
    distribution.distribution_end_timestamp = e.ledger().timestamp();
    distribution.distribution_total = total;
    distribution.distribution_treasury = treasury_amount;
    distribution.distribution_member = member_amount;
    distribution.is_processed = true;

    write_distribution(e, epoch, distribution);
    
    let mut distributions: Vec<u64> = read_all_epoch_distributions(e);
    
    distributions.push_back(epoch);

    write_distributions(e, distributions);

    // Increment epoch after distribution
    let next_epoch = epoch + 1;
    write_epoch(e, next_epoch);
    
    // Create the next distribution
    let next_distribution = Distribution { 
        distribution_start_timestamp: e.ledger().timestamp(),
        epoch: next_epoch,
        distribution_end_timestamp: 0,
        distribution_total: 0,
        distribution_treasury: 0,
        distribution_member: 0,
        members: get_active_members(e),
        is_processed: false,
    };
    write_distribution(e, next_epoch, next_distribution);
}

pub fn get_distribution_config(e: &Env) -> DistributionConfig { read_distribution_config(e) }

fn read_distribution_config(e: &Env) -> DistributionConfig {
    extend_instance(e);
    e.storage()
        .instance()
        .get(&DataKey::DistributionConfig)
        .unwrap()
}

fn write_distribution_config(e: &Env, config: DistributionConfig) {
    e.storage()
        .instance()
        .set(&DataKey::DistributionConfig, &config);
}

pub fn read_distribution_of_current_epoch(e: &Env) -> Distribution {
    extend_instance(e);
    let epoch = read_epoch_current(e);
    // Check if distribution exists for current epoch
    if e.storage().persistent().has(&DataKey::Distribution(epoch)) {
        read_distribution(e, epoch)
    } else {
        // Return a default distribution if none exists yet
        Distribution {
            distribution_start_timestamp: e.ledger().timestamp(),
            epoch: epoch,
            distribution_end_timestamp: 0,
            distribution_total: 0,
            distribution_treasury: 0,
            distribution_member: 0,
            members: Vec::new(e),
            is_processed: false,
        }
    }
}

pub fn read_distribution(e: &Env, epoch: u64) -> Distribution {
    extend_persistent(e, &DataKey::Distribution(epoch));
    e.storage()
        .persistent()
        .get(&DataKey::Distribution(epoch))
        .unwrap()
}

pub fn write_epoch(e: &Env, epoch: u64) {
    extend_instance(e);
    e.storage()
        .instance()
        .set(&CURRENT_EPOCH_KEY, &epoch);
}

pub fn read_next_distribution(e: &Env) -> u64 {
    // Check if config exists first
    match e.storage().instance().get::<DataKey, DistributionConfig>(&DataKey::DistributionConfig) {
        Some(config) => {
            let current_epoch = read_epoch_current(e);
            // Check if we have a distribution for the current epoch
            if e.storage().persistent().has(&DataKey::Distribution(current_epoch)) {
                let current_distribution = read_distribution(e, current_epoch);
                let next_time = current_distribution.distribution_start_timestamp + config.distribution_period;
                next_time
            } else {
                // No distribution exists yet, return current timestamp (immediately available)
                e.ledger().timestamp()
            }
        },
        None => {
            // Return current timestamp if no config exists yet
            e.ledger().timestamp()
        }
    }
}

pub fn check_distribution_availability(e: &Env) -> bool {
    // Check if config exists first
    if e.storage().instance().get::<DataKey, DistributionConfig>(&DataKey::DistributionConfig).is_none() {
        return false;
    }
    let current_epoch = read_epoch_current(e);
    // Check if distribution exists for current epoch
    if !e.storage().persistent().has(&DataKey::Distribution(current_epoch)) {
        return true; // First distribution is immediately available
    }
    let current_time = e.ledger().timestamp();
    let config = get_distribution_config(e);
    let current_distribution = read_distribution(e, current_epoch);
    
    // For the first distribution (epoch 0), it's immediately available
    if current_epoch == 0 && !current_distribution.is_processed {
        return true;
    }
    
    current_time >= (current_distribution.distribution_start_timestamp + config.distribution_period)
}

pub fn read_distribution_history(e: &Env) -> Vec<Distribution> {
    extend_instance(e);
    let distributions = read_all_epoch_distributions(e);
    let mut distributions_vec = Vec::new(e);
    for epoch in distributions.iter() {
        distributions_vec.push_back(read_distribution(e, epoch));
    }
    distributions_vec
}

fn read_all_epoch_distributions(e: &Env) -> Vec<u64> {
    match e.storage().persistent().get(&DataKey::Distributions) {
        Some(existing) => {
            extend_persistent(e, &DataKey::Distributions);
            existing
        },
        None => Vec::new(e),
    }
}

fn write_distributions(e: &Env, distributions: Vec<u64>) {
    e.storage()
        .persistent()
        .set(&DataKey::Distributions, &distributions);
    extend_persistent(e, &DataKey::Distributions);
}

fn write_distribution(e: &Env, epoch: u64, distribution: Distribution) {
    e.storage()
        .persistent()
        .set(&DataKey::Distribution(epoch), &distribution);
    extend_persistent(e, &DataKey::Distribution(epoch));
}

fn read_epoch_current(e: &Env) -> u64 {
    extend_instance(e);
    e.storage()
        .instance()
        .get(&CURRENT_EPOCH_KEY)
        .unwrap_or(0)
}
