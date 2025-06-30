use soroban_sdk::{Address, Env}; 

use crate::storage_types::{
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD, DataKey, PERSISTENT_BUMP_AMOUNT, PERSISTENT_LIFETIME_THRESHOLD
};

pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn extend_persistent(e: &Env, key: &DataKey) {
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
pub fn increase_cusd_supply(e: &Env, amount: &i128) {
    let cusd_supply = read_cusd_supply(e);
    write_cusd_supply(e, cusd_supply + amount);
}
pub fn decrease_cusd_supply(e: &Env, amount: &i128) {
    let cusd_supply = read_cusd_supply(e);
    write_cusd_supply(e, cusd_supply - amount); // we do not need to check for underflow since its not possible for the supply to be less than 0
}

pub fn read_cusd_total_supply(e: &Env) -> i128 { read_cusd_supply(e) }

fn read_cusd_supply(e: &Env) -> i128 {
    extend_persistent(e, &DataKey::CusdSupply);
    e.storage().persistent().get(&DataKey::CusdSupply).unwrap() // no need to handle if no key set since we initialize it to 0
}
fn write_cusd_supply(e: &Env, amount: i128) {
    extend_persistent(e, &DataKey::CusdSupply);
    e.storage().persistent().set(&DataKey::CusdSupply, &amount);
}

pub fn read_cusd_id(e: &Env) -> Address { read_address(e, &DataKey::Cusd) }
pub fn read_yield_controller(e: &Env) -> Address { read_address(e, &DataKey::YieldController) }
pub fn write_admin(e: &Env, new_admin: Address) { write_address(e, &DataKey::Admin, &new_admin);}
pub fn write_owner(e: &Env, new_owner: Address) { write_address(e, &DataKey::Owner, &new_owner);}
pub fn write_cusd(e: &Env, new_cusd: Address) { write_address(e, &DataKey::Cusd, &new_cusd);}
pub fn write_yield_controller(e: &Env, new_controller: Address) { write_address(e, &DataKey::YieldController, &new_controller);}
