use soroban_sdk::{Address, Env}; 

use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, DataKey};
fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
pub fn read_owner(e: &Env) -> Address {
    extend_instance(e);
    e.storage().instance().get(&DataKey::Owner).unwrap()
}

pub fn read_admin(e: &Env) -> Address {
    extend_instance(e);
    e.storage().instance().get(&DataKey::Admin).unwrap()
}
pub fn write_admin(e: &Env, new_admin: Address) { write_address(e, &DataKey::Admin, &new_admin);}
pub fn write_owner(e: &Env, new_owner: Address) { write_address(e, &DataKey::Owner, &new_owner);}

pub fn read_cusd_manager(e: &Env) -> Address {
    extend_instance(e);
    e.storage().instance().get(&DataKey::CUSDManager).unwrap()
}
pub fn write_cusd_manager(e: &Env, new_manager: Address) { write_address(e, &DataKey::CUSDManager, &new_manager);}

fn write_address(e: &Env, key: &DataKey, address: &Address) {
    extend_instance(e);
    e.storage().instance().set(key, address); 
}