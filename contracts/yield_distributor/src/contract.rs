use soroban_sdk::{contract, contractimpl, contractmeta, token::TokenClient, Address, Env, Vec};

use access_control::{access::default_access_control, constants::DEFAULT_ADMIN_ROLE};

use crate::events::YieldDistributorEvents;
use crate::storage::{
    add_member, get_active_members, get_distribution_config, get_distribution_period, get_member,
    get_treasury, get_treasury_share_bps, get_yield_controller, record_distribution, remove_member,
    set_distribution_period, set_treasury, set_treasury_share_bps, set_yield_controller,
};

contractmeta!(
    key = "Description",
    val = "Yield distributor for Coopstable"
);

fn require_admin(e: &Env, caller: &Address) {
    let access_control = default_access_control(e);
    access_control.only_role(e, caller, DEFAULT_ADMIN_ROLE);
}

fn require_yield_controller(e: &Env, caller: &Address) {
    let controller = get_yield_controller(e);
    if caller != &controller {
        panic!("Only the yield controller can call this function");
    }
}

pub trait YieldDistributorTrait {
    fn __constructor(
        e: Env,
        admin: Address,
        treasury: Address,
        treasury_share_bps: u32,
        yield_controller: Address,
        distribution_period: u64,
    );
    fn set_yield_controller(e: &Env, caller: Address, yield_controller: Address);
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
}

#[contract]
pub struct YieldDistributor;

#[contractimpl]
impl YieldDistributorTrait for YieldDistributor {
    fn __constructor(
        e: Env,
        admin: Address,
        treasury: Address,
        treasury_share_bps: u32,
        yield_controller: Address,
        distribution_period: u64,
    ) {
        let access_control = default_access_control(&e);

        access_control.initialize(&e, &admin);

        set_treasury(&e, &treasury);
        set_treasury_share_bps(&e, treasury_share_bps);
        set_yield_controller(&e, &yield_controller);
        set_distribution_period(&e, distribution_period);
    }

    fn set_yield_controller(e: &Env, caller: Address, yield_controller: Address) {
        require_admin(e, &caller);
        set_yield_controller(e, &yield_controller);
        YieldDistributorEvents::set_yield_controller(&e, yield_controller);
    }

    fn add_member(e: &Env, caller: Address, member: Address) {
        require_admin(e, &caller);

        if let Some(existing) = get_member(e, &member) {
            if existing.active {
                panic!("Member already exists and is active");
            }
        }

        add_member(e, &member);
        YieldDistributorEvents::add_member(e, member);
    }

    fn remove_member(e: &Env, caller: Address, member: Address) {
        require_admin(e, &caller);

        if let None = get_member(e, &member) {
            panic!("Member does not exist");
        }

        remove_member(e, &member);
        YieldDistributorEvents::remove_member(e, member);
    }

    fn list_members(e: &Env) -> Vec<Address> {
        get_active_members(e)
    }

    fn set_treasury(e: &Env, caller: Address, treasury: Address) {
        require_admin(e, &caller);
        set_treasury(e, &treasury);
        YieldDistributorEvents::set_treasury(e, treasury);
    }

    fn get_treasury(e: &Env) -> Address {
        get_treasury(e)
    }

    fn set_treasury_share(e: &Env, caller: Address, share_bps: u32) {
        require_admin(e, &caller);
        set_treasury_share_bps(e, share_bps);
        YieldDistributorEvents::set_treasury_share(e, share_bps);
    }

    fn get_treasury_share(e: &Env) -> u32 {
        get_treasury_share_bps(e)
    }

    fn set_distribution_period(e: &Env, caller: Address, period: u64) {
        require_admin(e, &caller);
        set_distribution_period(e, period);
        YieldDistributorEvents::set_distribution_period(e, period);
    }

    fn get_distribution_period(e: &Env) -> u64 {
        get_distribution_period(e)
    }

    fn get_next_distribution_time(e: &Env) -> u64 {
        let config = get_distribution_config(e);
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

        let members = get_active_members(e);
        let member_count = members.len() as u32;

        if member_count == 0 {
            return false;
        }

        let treasury_share_bps = get_treasury_share_bps(e);
        let treasury = get_treasury(e);

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

        record_distribution(e, amount, treasury_amount, members_amount);

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
}
