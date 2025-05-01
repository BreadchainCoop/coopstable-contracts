use soroban_sdk::{Address, Env};

use crate::storage_types::{
    DataKey,
    INSTANCE_BUMP_AMOUNT, 
    INSTANCE_LIFETIME_THRESHOLD
};

pub fn read_cusd_manager_admin(e: &Env) -> Address {
    let key = DataKey::Manager;

    e.storage()
    .instance()
    .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    e.storage().instance().get(&key).unwrap()
}

pub fn write_cusd_manager_admin(e: &Env, id: &Address) {
    let key = DataKey::Manager;

    e.storage()
    .instance()
    .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    e.storage().instance().set(&key, id);
}
