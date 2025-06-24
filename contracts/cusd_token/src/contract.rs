use soroban_sdk::{Address, contract, contractimpl, Env, String, contractmeta};
use stellar_fungible::{ self as fungible, burnable::FungibleBurnable, FungibleToken };
use crate::{ events::CUSDEvents, storage };

contractmeta!(
    key = "Description",
    val = "Stellar asset contract for the Coopstable cUSD system"
);

const DECIMALS: u32 = 7;

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }
fn require_cusd_manager(e: &Env) { storage::read_cusd_manager(e).require_auth(); }

#[contract]
pub struct CUSD;

#[contractimpl]
impl CUSD {
    pub fn __constructor(e: &Env, owner: Address, cusd_manager: Address, admin: Address) {
        fungible::metadata::set_metadata(e, DECIMALS, String::from_str(e, "cUSD"), String::from_str(e, "CUSD"));    
        storage::write_owner(e, owner);
        storage::write_cusd_manager(e, cusd_manager);
        storage::write_admin(e, admin);
    }
}

#[contractimpl]
impl FungibleToken for CUSD {
    fn total_supply(e: &Env) -> i128 {
        fungible::total_supply(e)
    }

    fn balance(e: &Env, account: Address) -> i128 {
        fungible::balance(e, &account)
    }

    fn allowance(e: &Env, owner: Address, spender: Address) -> i128 {
        fungible::allowance(e, &owner, &spender)
    }

    fn transfer(e: &Env, from: Address, to: Address, amount: i128) {
        fungible::transfer(e, &from, &to, amount);
    }

    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, amount: i128) {
        fungible::transfer_from(e, &spender, &from, &to, amount);
    }

    fn approve(e: &Env, owner: Address, spender: Address, amount: i128, live_until_ledger: u32) {
        fungible::approve(e, &owner, &spender, amount, live_until_ledger);
    }

    fn decimals(e: &Env) -> u32 {
        fungible::metadata::decimals(e)
    }

    fn name(e: &Env) -> String {
        fungible::metadata::name(e)
    }

    fn symbol(e: &Env) -> String {
        fungible::metadata::symbol(e)
    }
}

#[contractimpl]
impl FungibleBurnable for CUSD {
    fn burn(e: &Env, from: Address, amount: i128) {
        fungible::burnable::burn(e, &from, amount);
    }
    fn burn_from(e: &Env, spender: Address, from: Address, amount: i128) {
        fungible::burnable::burn_from(e, &spender, &from, amount);
    }
}

pub trait FungibleMintableAccessControll {
    fn mint(e: &Env, to: Address, amount: i128);
}

#[contractimpl]
impl FungibleMintableAccessControll for CUSD {
    fn mint(e: &Env, to: Address, amount: i128) {
        require_cusd_manager(e);
        fungible::mintable::mint(e, &to, amount);
    }
}

pub trait FungibleAdmin {
    fn set_cusd_manager(e: &Env, new_manager: Address);
    fn set_admin(e: &Env, new_admin: Address);
}

#[contractimpl]
impl FungibleAdmin for CUSD {
    fn set_cusd_manager(e: &Env, new_manager: Address) {
        require_admin(e);
        storage::write_cusd_manager(e, new_manager.clone());
        CUSDEvents::set_cusd_manager(&e, new_manager);
    }
    fn set_admin(e: &Env, new_admin: Address) {
        require_owner(e);
        storage::write_admin(e, new_admin.clone()); 
        CUSDEvents::set_admin(&e, new_admin);
    }
}
