use crate::events::CUSDManagerEvents;
use crate::error::CUSDManagerError;
use crate::storage_types::DataKey;
use crate::token;
use crate::storage;
use soroban_sdk::{ contract, contractimpl, contractmeta, Address, Env, panic_with_error };

contractmeta!(
    key = "Description",
    val = "Coopstable cUSD manager steward of the cusd token"
);

fn check_nonnegative_amount(e: &Env, amount: i128) {
    if amount < 0 {
        panic_with_error!(e, CUSDManagerError::NegativeAmountError);
    }
}

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }

#[contract]
pub struct CUSDManager;

pub trait CUSDManagerTrait {
    fn __constructor(e: Env, cusd_id: Address, owner: Address, admin: Address);
    fn issue_cusd(e: &Env, to: Address, amount: i128);
    fn burn_cusd(e: &Env, from: Address, amount: i128);
    fn get_cusd_id(e: &Env) -> Address;
    fn cusd_total_supply(e: &Env) -> i128;
    fn set_admin(e: &Env, new_admin: Address); 
    fn set_yield_controller(e: &Env, new_controller: Address);
    fn set_cusd_id(e: &Env, new_cusd_id: Address);
    fn set_cusd_issuer(e: &Env, new_issuer: Address);
}

#[contractimpl]
impl CUSDManagerTrait for CUSDManager {
    fn __constructor(e: Env, cusd_id: Address, owner: Address, admin: Address) {
        storage::write_owner(&e, owner);
        storage::write_admin(&e, admin);
        storage::write_cusd(&e, cusd_id); 

        e.storage().persistent().set(&DataKey::CusdSupply, &0i128);
    }
    
    fn set_admin(e: &Env, new_admin: Address) {
        require_owner(e);
        storage::write_admin(e, new_admin.clone());
        CUSDManagerEvents::set_admin(&e, new_admin);
    }

    fn get_cusd_id(e: &Env) -> Address {
        storage::read_cusd_id(e)
    }

    fn issue_cusd(e: &Env, to: Address, amount: i128) {
        storage::read_yield_controller(e).require_auth();
        check_nonnegative_amount(e, amount);
        token::process_token_mint(&e, to.clone(), amount);
        CUSDManagerEvents::issue_cusd(&e, to, amount);
    }

    fn set_cusd_id(e: &Env, new_cusd_id: Address) {
        require_admin(e);
        storage::write_cusd(e, new_cusd_id.clone());
        CUSDManagerEvents::set_cusd_id(&e, new_cusd_id);
    }

    fn set_yield_controller(e: &Env, new_controller: Address) {
        require_admin(e);
        storage::write_yield_controller(e, new_controller.clone());
        CUSDManagerEvents::set_yield_controller(&e, new_controller);
    }

    fn burn_cusd(e: &Env, from: Address, amount: i128) {
        check_nonnegative_amount(e, amount);
        token::process_token_burn(&e, from.clone(), amount);
        CUSDManagerEvents::burn_cusd(&e, from, amount);
    }

    fn set_cusd_issuer(e: &Env, new_issuer: Address) {
        require_admin(e);
        token::set_issuer(&e, &storage::read_cusd_id(&e), &new_issuer);
        CUSDManagerEvents::set_cusd_issuer(&e, new_issuer);
    }

    fn cusd_total_supply(e: &Env) -> i128 { storage::read_cusd_total_supply(&e) }
}
