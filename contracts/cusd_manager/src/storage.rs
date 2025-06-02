use soroban_sdk::{Address, Env}; 

use crate::storage_types::{
    CUSD_ADDRESS_KEY,
    INSTANCE_BUMP_AMOUNT, 
    INSTANCE_LIFETIME_THRESHOLD,
};

pub fn read_cusd_id(e: &Env) -> Address {
    e.storage()
    .instance()
    .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    e.storage().instance().get(&CUSD_ADDRESS_KEY).unwrap()
}