use soroban_sdk::{contract, contractimpl, contractmeta, token::TokenClient, Address, Env, Vec};
use access_control::{access::default_access_control, constants::DEFAULT_ADMIN_ROLE};

use crate::events::YieldDistributorEvents;
use crate::storage;
use crate::storage_types::YIELD_DISTRIBUTOR_ADMIN_ROLE;

contractmeta!(
    key = "Description",
    val = "Yield distributor for Coopstable"
);

fn require_admin(e: &Env, caller: &Address) {
    let access_control = default_access_control(e);
    access_control.only_role(e, caller, YIELD_DISTRIBUTOR_ADMIN_ROLE);
}

fn require_yield_controller(e: &Env, caller: &Address) {
    let controller = storage::get_yield_controller(e);
    if caller != &controller {
        panic!("Only the yield controller can call this function");
    }
}

pub trait YieldDistributorTrait {
    fn __constructor(
        e: Env,
        treasury: Address,
        treasury_share_bps: u32,
        yield_controller: Address,
        distribution_period: u64,
        owner: Address,
        admin: Address,
    );
    fn set_yield_controller(e: &Env, caller: Address, yield_controller: Address);
    fn get_yield_controller(e: &Env) -> Address;
    fn add_member(e: &Env, caller: Address, member: Address);
    fn remove_member(e: &Env, caller: Address, member: Address);
    fn list_members(e: &Env) -> Vec<Address>;

    fn set_treasury(e: &Env, caller: Address, treasury: Address);
    fn get_treasury(e: &Env) -> Address;

    fn set_treasury_share(e: &Env, caller: Address, share_bps: u32);
    fn get_treasury_share(e: &Env) -> u32;

    fn set_distribution_period(e: &Env, caller: Address, period: u64);
    fn get_distribution_period(e: &Env) -> u64;

    fn get_next_distribution_time(e: &Env) -> u64;
    fn is_distribution_available(e: &Env) -> bool;

    fn distribute_yield(e: &Env, caller: Address, token: Address, amount: i128) -> bool;

    fn set_admin(e: &Env, caller: Address, new_admin: Address);
    
}

#[contract]
pub struct YieldDistributor;

#[contractimpl]
impl YieldDistributorTrait for YieldDistributor {
    fn __constructor(
        e: Env,
        treasury: Address,
        treasury_share_bps: u32,
        yield_controller: Address,
        distribution_period: u64,
        owner: Address,
        admin: Address,
    ) {
        let access_control = default_access_control(&e);

        access_control.initialize(&e, &owner);
        access_control.set_role_admin(&e, YIELD_DISTRIBUTOR_ADMIN_ROLE, DEFAULT_ADMIN_ROLE);
        access_control._grant_role(&e, YIELD_DISTRIBUTOR_ADMIN_ROLE, &admin);
        
        storage::set_treasury(&e, &treasury);
        storage::set_treasury_share_bps(&e, treasury_share_bps);
        storage::set_yield_controller(&e, &yield_controller);
        storage::set_distribution_period(&e, distribution_period);
    }

    fn set_yield_controller(e: &Env, caller: Address, yield_controller: Address) {
        require_admin(e, &caller);
        storage::set_yield_controller(e, &yield_controller);
        YieldDistributorEvents::set_yield_controller(&e, yield_controller);
    }

    fn add_member(e: &Env, caller: Address, member: Address) {
        require_admin(e, &caller);

        if let Some(existing) = storage::get_member(e, &member) {
            if existing.active {
                panic!("Member already exists and is active");
            }
        }

        storage::add_member(e, &member);
        YieldDistributorEvents::add_member(e, member);
    }

    fn remove_member(e: &Env, caller: Address, member: Address) {
        require_admin(e, &caller);

        if let None = storage::get_member(e, &member) {
            panic!("Member does not exist");
        }

        storage::remove_member(e, &member);
        YieldDistributorEvents::remove_member(e, member);
    }

    fn list_members(e: &Env) -> Vec<Address> {
        storage::get_active_members(e)
    }

    fn set_treasury(e: &Env, caller: Address, treasury: Address) {
        require_admin(e, &caller);
        storage::set_treasury(e, &treasury);
        YieldDistributorEvents::set_treasury(e, treasury);
    }

    fn get_treasury(e: &Env) -> Address {
        storage::get_treasury(e)
    }

    fn set_treasury_share(e: &Env, caller: Address, share_bps: u32) {
        require_admin(e, &caller);
        storage::set_treasury_share_bps(e, share_bps);
        YieldDistributorEvents::set_treasury_share(e, share_bps);
    }

    fn get_treasury_share(e: &Env) -> u32 {
        storage::get_treasury_share_bps(e)
    }

    fn set_distribution_period(e: &Env, caller: Address, period: u64) {
        require_admin(e, &caller);
        storage::set_distribution_period(e, period);
        YieldDistributorEvents::set_distribution_period(e, period);
    }

    fn get_distribution_period(e: &Env) -> u64 {
        storage::get_distribution_period(e)
    }

    fn get_next_distribution_time(e: &Env) -> u64 {
        let config = storage::get_distribution_config(e);
        let next_time = config.last_distribution + config.distribution_period;
        next_time
    }

    fn is_distribution_available(e: &Env) -> bool {
        let current_time = e.ledger().timestamp();
        let next_time = Self::get_next_distribution_time(e);
        current_time >= next_time
    }

    fn distribute_yield(e: &Env, caller: Address, token: Address, amount: i128) -> bool {
        require_yield_controller(e, &caller);

        if !Self::is_distribution_available(e) {
            return false;
        }

        let members = storage::get_active_members(e);
        let member_count = members.len() as u32;

        if member_count == 0 {
            return false;
        }

        let treasury_share_bps = storage::get_treasury_share_bps(e);
        let treasury = storage::get_treasury(e);

        let treasury_amount = (amount as i128 * treasury_share_bps as i128) / 10000;
        let members_amount = amount - treasury_amount;

        let per_member_amount = if member_count > 0 {
            members_amount / member_count as i128
        } else {
            0
        };

        let token_client = TokenClient::new(e, &token);

        if treasury_amount > 0 {
            token_client.transfer_from(
                &e.current_contract_address(),
                &caller,
                &treasury,
                &treasury_amount,
            );
        }

        if per_member_amount > 0 {
            for member in members.iter() {
                token_client.transfer_from(
                    &e.current_contract_address(),
                    &caller,
                    &member,
                    &per_member_amount,
                );
            }
        }

        storage::record_distribution(e, amount, treasury_amount, members_amount);

        YieldDistributorEvents::distribute_yield(
            e,
            token,
            amount,
            treasury_amount,
            members,
            per_member_amount,
        );

        true
    }

    fn set_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(&e, caller, YIELD_DISTRIBUTOR_ADMIN_ROLE, &new_admin);
        YieldDistributorEvents::set_admin(&e, new_admin);
    }

    fn get_yield_controller(e: &Env) -> Address {
        storage::get_yield_controller(e)
    }
}
