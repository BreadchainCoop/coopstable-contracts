use soroban_sdk::{Address, Env};

use crate::storage_types::DataKey;

pub fn read_cusd_manager(e: &Env) -> Address {
    let key = DataKey::Manager;
    e.storage().instance().get(&key).unwrap()
}

pub fn write_cusd_manager(e: &Env, id: &Address) {
    let key = DataKey::Manager;
    e.storage().instance().set(&key, id);
}
