use soroban_sdk::{
    contract, 
    contractclient, 
    contractimpl, 
    contractmeta, 
    symbol_short, 
    Address, 
    Env, 
    Symbol,
    token::StellarAssetClient
};

use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::token::{process_token_burn, process_token_mint};
use access_control::{
    access::default_access_control,
    constants::DEFAULT_ADMIN_ROLE
};

const CUSD_ADMIN: Symbol = symbol_short!("CUSD_ADMN");
const CUSD_ADDRESS_KEY: Symbol = symbol_short!("cUSD");

contractmeta!(key = "Description", val = "Coopstable cUSD manager contract");

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

#[contract]
pub struct CusdManager;

// manager manages collateral assets supported by this contract
#[contractclient(name = "CusdManagerAdminClient")]
pub trait CusdManagerAdmin {
    fn set_default_admin(e: &Env, caller: Address, new_admin: Address);
    fn set_admin(e: &Env, caller: Address, new_manager: Address);
    fn only_admin(e: &Env, caller: Address);
    fn set_cusd_issuer(e: &Env, caller: Address, new_issuer: Address);
}

#[contractclient(name = "CusdManagerTokenClient")]
pub trait CusdManagerToken {
    fn issue_cusd(e: &Env, caller: Address, to: Address, amount: i128);
    fn burn_cusd(e: &Env, caller: Address, from: Address, amount: i128);
    fn get_cusd_id(e: &Env) -> Address;
}

#[contractimpl]
impl CusdManager {
    fn __constructor(
        e: Env, 
        cusd_id: Address,
        owner: Address, 
        admin: Address
    ) {
        let access_control = default_access_control(&e);

        access_control.initialize(&e, &owner);
        access_control.set_role_admin(&e, CUSD_ADMIN, DEFAULT_ADMIN_ROLE); 
        access_control._grant_role(&e, CUSD_ADMIN, &admin);

        e.storage()
            .instance()
            .set(&CUSD_ADDRESS_KEY, &cusd_id);
    }
}

#[contractimpl]
impl CusdManagerAdmin for CusdManager {

    fn set_default_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(&e, caller, DEFAULT_ADMIN_ROLE, &new_admin);
    }

    fn set_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(&e, caller, CUSD_ADMIN, &new_admin);
    }

    fn only_admin(e: &Env, caller: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(&e, &caller, CUSD_ADMIN);
    }

    fn set_cusd_issuer(e: &Env, caller: Address, new_issuer: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(&e, &caller, CUSD_ADMIN);
        let token_admin_client = StellarAssetClient::new(&e, &Self::get_cusd_id(&e));
        token_admin_client.set_admin(&new_issuer);        
    }
}


#[contractimpl]
impl CusdManagerToken for CusdManager {
    
    fn get_cusd_id(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD);
        
        e.storage().instance().get(&CUSD_ADDRESS_KEY).unwrap()
    }

    fn issue_cusd(e: &Env, caller: Address, to: Address, amount: i128) {
        Self::only_admin(e, caller);
        check_nonnegative_amount(amount);
        process_token_mint(&e, to.clone(), Self::get_cusd_id(&e), amount); 
        e.events().publish(("CUSD_MANAGER", "mint_cusd"), to);
    }

    fn burn_cusd(e: &Env, caller: Address, from: Address, amount: i128) {
        Self::only_admin(e, caller);
        check_nonnegative_amount(amount);
        process_token_burn(&e, from.clone(), Self::get_cusd_id(&e), amount);
        e.events().publish(("CUSD_MANAGER", "burn_cusd"), from);
    }
}