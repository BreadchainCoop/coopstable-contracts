use soroban_sdk::{
    contract, 
    contractclient, 
    contractimpl, 
    contractmeta, 
    symbol_short, 
    Address, 
    Env, 
    Symbol
};

use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::admin::{read_administrator, write_administrator};
use crate::manager::{read_cusd_manager_admin, write_cusd_manager_admin};
use crate::token::{process_token_burn, process_token_mint};

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
    fn set_admin(e: Env, admin: Address);
    fn set_manager(e: Env, new_manager: Address);
}

#[contractclient(name = "CusdManagerTokenClient")]
pub trait CusdManagerToken {
    fn issue_cusd(e: Env, owner: Address, amount: i128);
    fn burn_cusd(e: Env, owner: Address, amount: i128);
    fn get_cusd_id(e: &Env) -> Address;
}

#[contractimpl]
impl CusdManager {
    fn __constructor(
        e: Env, 
        cusd_id: Address,
        admin: Address, 
        manager: Address
    ) {
        let admin = admin;

        write_administrator(&e, &admin);
        
        write_cusd_manager_admin(&e, &manager);

        e.storage()
            .instance()
            .set(&CUSD_SYMBOL, &cusd_id);
    }
}

#[contractimpl]
impl CusdManagerAdmin for CusdManager {

    fn set_admin(e: Env, new_admin: Address) {
        
        let admin = read_administrator(&e);
        
        admin.require_auth();

        write_administrator(&e, &new_admin);
        
        e.events().publish(("CUSD_MANAGER", "set_admin"), &new_admin);
    }

    fn set_manager(e: Env, new_manager: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();
        write_cusd_manager_admin(&e, &new_manager);
        e.events().publish(("CUSD_MANAGER", "set_manager"), new_manager);
    }
}

const CUSD_SYMBOL: Symbol = symbol_short!("cUSD");

#[contractimpl]
impl CusdManagerToken for CusdManager {
    
    fn get_cusd_id(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD);
        
        e.storage().instance().get(&CUSD_SYMBOL).unwrap()
    }

    fn issue_cusd(e: Env, owner: Address, amount: i128) {
        let manager = read_cusd_manager_admin(&e);
        manager.require_auth();
        check_nonnegative_amount(amount);
        process_token_mint(&e, owner.clone(), Self::get_cusd_id(&e), amount); 
        e.events().publish(("CUSD_MANAGER", "mint_cusd"), owner);
    }

    fn burn_cusd(e: Env, owner: Address, amount: i128) {
        let manager = read_cusd_manager_admin(&e);
        manager.require_auth();
        check_nonnegative_amount(amount);
        process_token_burn(&e, owner.clone(), Self::get_cusd_id(&e), amount);
        e.events().publish(("CUSD_MANAGER", "burn_cusd"), owner);
    }
}