use core::panic;
use soroban_sdk::{contract, contractimpl, contractmeta, token::TokenClient, Address, Env, Vec, panic_with_error};
use crate::events::YieldDistributorEvents;
use crate::error::YieldDistributorError;
use crate::storage;

contractmeta!(
    key = "Description",
    val = "Yield distributor contract for Coopstable"
);

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }

fn require_yield_controller(e: &Env ) { storage::get_yield_controller(e).require_auth(); }

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
    fn set_yield_controller(e: &Env, yield_controller: Address);
    fn get_yield_controller(e: &Env) -> Address;
    fn add_member(e: &Env, member: Address);
    fn remove_member(e: &Env, member: Address);
    fn list_members(e: &Env) -> Vec<Address>;

    fn set_treasury(e: &Env, treasury: Address);
    fn get_treasury(e: &Env) -> Address;

    fn set_treasury_share(e: &Env, share_bps: u32);
    fn get_treasury_share(e: &Env) -> u32;

    fn set_distribution_period(e: &Env, period: u64);
    fn get_distribution_period(e: &Env) -> u64;

    fn get_next_distribution_time(e: &Env) -> u64;
    fn is_distribution_available(e: &Env) -> bool;

    fn distribute_yield(e: &Env, token: Address, amount: i128) -> bool;

    fn set_admin(e: &Env, new_admin: Address);
    
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
        storage::write_admin(&e, admin);
        storage::write_owner(&e, owner);
        storage::set_treasury(&e, &treasury);
        storage::set_treasury_share_bps(&e, treasury_share_bps);
        storage::set_yield_controller(&e, &yield_controller);
        storage::set_distribution_period(&e, distribution_period);
    }

    fn set_yield_controller(e: &Env, yield_controller: Address) {
        require_admin(e);
        storage::set_yield_controller(e, &yield_controller);
        YieldDistributorEvents::set_yield_controller(&e, yield_controller);
    }

    fn add_member(e: &Env, member: Address) {
        require_admin(e);

        if let Some(existing) = storage::get_member(e, &member) {
            if existing.active {
                panic_with_error!(e, YieldDistributorError::MemberAlreadyExists);
            }
        }

        storage::add_member(e, &member);
        YieldDistributorEvents::add_member(e, member);
    }

    fn remove_member(e: &Env, member: Address) {
        require_admin(e);

        if let None = storage::get_member(e, &member) {
            panic_with_error!(e, YieldDistributorError::MemberDoesNotExist);
        }

        storage::remove_member(e, &member);
        YieldDistributorEvents::remove_member(e, member);
    }

    fn list_members(e: &Env) -> Vec<Address> {
        storage::get_active_members(e)
    }

    fn set_treasury(e: &Env, treasury: Address) {
        require_admin(e);
        storage::set_treasury(e, &treasury);
        YieldDistributorEvents::set_treasury(e, treasury);
    }

    fn get_treasury(e: &Env) -> Address {
        storage::get_treasury(e)
    }

    fn set_treasury_share(e: &Env, share_bps: u32) {
        require_admin(e);
        storage::set_treasury_share_bps(e, share_bps);
        YieldDistributorEvents::set_treasury_share(e, share_bps);
    }

    fn get_treasury_share(e: &Env) -> u32 {
        storage::get_treasury_share_bps(e)
    }

    fn set_distribution_period(e: &Env, period: u64) {
        require_admin(e);
        storage::set_distribution_period(e, period);
        YieldDistributorEvents::set_distribution_period(e, period);
    }

    fn get_distribution_period(e: &Env) -> u64 {
        storage::get_distribution_period(e)
    }

    fn get_next_distribution_time(e: &Env) -> u64 {
        storage::read_next_distribution(e)
    }

    fn is_distribution_available(e: &Env) -> bool {
        storage::check_distribution_availability(e)
    }

    fn distribute_yield(e: &Env, token: Address, amount: i128) -> bool {
        require_yield_controller(e);

        if !storage::check_distribution_availability(e) {
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
            token_client.transfer(
                &storage::get_yield_controller(e),
                &treasury,
                &treasury_amount,
            );
        }

        if per_member_amount > 0 {
            for member in members.iter() {
                token_client.transfer(
                    &storage::get_yield_controller(e),
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

    fn set_admin(e: &Env, new_admin: Address) {
        require_owner(e);
        YieldDistributorEvents::set_admin(&e, new_admin);
    }

    fn get_yield_controller(e: &Env) -> Address {
        storage::get_yield_controller(e)
    }
}
