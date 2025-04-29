use soroban_sdk::{Address, Env, Symbol, symbol_short};
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};

pub(crate) const ADMIN_KEY: Symbol = symbol_short!("ADMIN");

pub fn read_administrator(e: &Env) -> Address {
    e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    e.storage().instance().get(&ADMIN_KEY).unwrap()
}

pub fn write_administrator(e: &Env, id: &Address) {
    e.storage().instance().extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
    e.storage().instance().set(&ADMIN_KEY, id);
}

pub fn require_admin(e: &Env) {
    let admin = read_administrator(e);
    admin.require_auth();
}
