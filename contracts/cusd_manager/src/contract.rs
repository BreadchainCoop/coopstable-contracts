use crate::events::CUSDManagerEvents;
use crate::error::CUSDManagerError;
use crate::storage_types::{
    CUSD_ADDRESS_KEY, 
    CUSD_ADMIN, 
    INSTANCE_BUMP_AMOUNT, 
    INSTANCE_LIFETIME_THRESHOLD,
    YIELD_CONTROLLER
};
use crate::token::{ensure_sufficient_balance, process_token_burn, process_token_mint};
use access_control::access;
use access_control::{access::default_access_control, constants::DEFAULT_ADMIN_ROLE};
use soroban_sdk::{
    contract, contractimpl, contractmeta, token::StellarAssetClient, Address, Env, panic_with_error
};


contractmeta!(
    key = "Description",
    val = "Coopstable cUSD manager contract"
);

fn check_nonnegative_amount(e: &Env, amount: i128) {
    if amount < 0 {
        panic_with_error!(e, CUSDManagerError::NegativeAmountError);
    }
}

fn only_admin(e: &Env, caller: Address) {
    let access_control = default_access_control(e);
    access_control.only_role(&e, &caller, CUSD_ADMIN);
}

#[contract]
pub struct CUSDManager;

pub trait CUSDManagerTrait {
    fn __constructor(e: Env, cusd_id: Address, owner: Address, admin: Address);
    fn set_default_admin(e: &Env, caller: Address, new_admin: Address);
    fn set_cusd_manager_admin(e: &Env, caller: Address, new_manager: Address);
    fn set_cusd_issuer(e: &Env, caller: Address, new_issuer: Address);
    fn issue_cusd(e: &Env, caller: Address, to: Address, amount: i128);
    fn burn_cusd(e: &Env, caller: Address, from: Address, amount: i128);
    fn get_cusd_id(e: &Env) -> Address;
    fn set_yield_controller(e: &Env, caller: Address, new_controller: Address);
}

#[contractimpl]
impl CUSDManagerTrait for CUSDManager {
    fn __constructor(e: Env, cusd_id: Address, owner: Address, admin: Address) {
        let access_control = default_access_control(&e);

        access_control.initialize(&e, &owner);
        access_control.set_role_admin(&e, CUSD_ADMIN, DEFAULT_ADMIN_ROLE);
        access_control._grant_role(&e, CUSD_ADMIN, &admin);
        access_control.set_role_admin(&e, YIELD_CONTROLLER, CUSD_ADMIN);

        e.storage().instance().set(&CUSD_ADDRESS_KEY, &cusd_id);
    }
    fn set_default_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(&e, caller, DEFAULT_ADMIN_ROLE, &new_admin);
    }

    fn set_cusd_manager_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(&e, caller, CUSD_ADMIN, &new_admin);
        CUSDManagerEvents::set_cusd_manager_admin(&e, new_admin);
    }

    fn set_cusd_issuer(e: &Env, caller: Address, new_issuer: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(&e, &caller, DEFAULT_ADMIN_ROLE);

        let token_admin_client = StellarAssetClient::new(&e, &Self::get_cusd_id(&e));
        token_admin_client.set_admin(&new_issuer);
        CUSDManagerEvents::set_cusd_issuer(&e, new_issuer);
    }

    fn get_cusd_id(e: &Env) -> Address {
        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        e.storage().instance().get(&CUSD_ADDRESS_KEY).unwrap()
    }

    fn issue_cusd(e: &Env, caller: Address, to: Address, amount: i128) {
        access::default_access_control(e).only_role(e, &caller, YIELD_CONTROLLER);
        check_nonnegative_amount(e, amount);
        process_token_mint(&e, to.clone(), Self::get_cusd_id(&e), amount);
        CUSDManagerEvents::issue_cusd(&e, to, amount);
    }

    fn set_yield_controller(e: &Env, caller: Address, new_controller: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(e, caller, YIELD_CONTROLLER, &new_controller); 
        
        CUSDManagerEvents::set_yield_controller(&e, new_controller);
    }

    fn burn_cusd(e: &Env, caller: Address, from: Address, amount: i128) {
        access::default_access_control(e).only_role(e, &caller, YIELD_CONTROLLER);
        check_nonnegative_amount(e, amount);
        ensure_sufficient_balance(
            e, 
            e.current_contract_address(), 
            Self::get_cusd_id(&e), 
            amount
        );
        process_token_burn(
            &e,
            e.current_contract_address(),
            Self::get_cusd_id(&e),
            amount,
        );
        CUSDManagerEvents::burn_cusd(&e, from, amount);
    }
}
