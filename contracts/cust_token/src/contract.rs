use soroban_sdk::{Address, contract, contractimpl, Env, String, Symbol, symbol_short};
use stellar_fungible::{self as fungible, burnable::FungibleBurnable, FungibleToken, mintable::FungibleMintable};
use access_control::{access, constants::DEFAULT_ADMIN_ROLE};

const OWNER: Symbol = symbol_short!("OWNER");
const TOKEN_ADMIN: Symbol = symbol_short!("TKADM");
const CUSD_MANAGER: Symbol = symbol_short!("CSDM");
const DECIMALS: u32 = 7;

#[contract]
pub struct CUSD;

#[contractimpl]
impl CUSD {
    pub fn __constructor(e: &Env, owner: Address, admin: Address) {
        let access_control = access::default_access_control(&e);
        fungible::metadata::set_metadata(e, DECIMALS, String::from_str(e, "cUSD"), String::from_str(e, "CUSD"));
        
        access_control.initialize(&e, &owner);
        access_control.set_role_admin(&e, TOKEN_ADMIN, DEFAULT_ADMIN_ROLE);
        access_control._grant_role(&e, TOKEN_ADMIN, &admin);
        
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

#[contractimpl]
impl FungibleMintable for CUSD {
    fn mint(e: &Env, account: Address, amount: i128) {
        let owner: Address = e.storage().instance().get(&OWNER).expect("owner should be set");
        owner.require_auth();
        fungible::mintable::mint(e, &account, amount);
    }
}

pub trait FungibleAdmin {
    fn set_admin(e: &Env, caller: Address, new_admin: Address);
    fn set_manager(e: &Env, caller: Address, new_manager: Address);
}

#[contractimpl]
impl FungibleAdmin for CUSD {
    fn set_admin(e: &Env, caller: Address, new_admin: Address) {
        access::default_access_control(e).grant_role(&e, caller, TOKEN_ADMIN, &new_admin);
        CUSDEvents::set_admin(&e, new_admin);
    }

    fn set_manager(e: &Env, caller: Address, new_manager: Address) {
        access::default_access_control(e).only_role(e, &caller, TOKEN_ADMIN);
        e.storage().instance().set(&CUSD_MANAGER, &new_manager);
        CUSDEvents::set_manager(&e, new_manager);
    }
}
