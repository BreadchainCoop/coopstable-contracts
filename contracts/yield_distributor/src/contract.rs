use soroban_sdk::{vec, IntoVal, Symbol, contract, contractimpl, contractmeta, token::TokenClient, Address, BytesN, Env, Vec, panic_with_error};
use crate::events::YieldDistributorEvents;
use crate::error::YieldDistributorError;
use crate::storage_types::Distribution;
use crate::{storage, storage_types, utils};

contractmeta!(
    key = "Description",
    val = "Yield distributor contract for Coopstable"
);

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }

fn require_yield_controller(e: &Env ) { storage::get_yield_controller(e).require_auth(); }

pub trait YieldDistributorTrait {
    /// Initialize the Yield Distributor contract
    ///
    /// ### Arguments
    /// * `treasury` - The address of the treasury that receives a share of yield
    /// * `treasury_share_bps` - The treasury's share in basis points (1-10000)
    /// * `yield_controller` - The address of the yield controller contract
    /// * `distribution_period` - The time period between distributions (in seconds)
    /// * `owner` - The address of the contract owner (can set admin)
    /// * `admin` - The address of the admin (manages members and settings)
    fn __constructor(
        e: Env,
        treasury: Address,
        treasury_share_bps: u32,
        yield_controller: Address,
        distribution_period: u64,
        owner: Address,
        admin: Address,
    );
    
    /// (Admin only) Set a new yield controller address
    ///
    /// ### Arguments
    /// * `yield_controller` - The new yield controller contract address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_yield_controller(e: &Env, yield_controller: Address);
    
    /// Fetch the address of the yield controller contract
    fn get_yield_controller(e: &Env) -> Address;
    
    /// (Admin only) Add a member to receive yield distributions
    ///
    /// ### Arguments
    /// * `member` - The address of the member to add
    ///
    /// ### Panics
    /// If the caller is not the admin
    /// If the member already exists and is active
    fn add_member(e: &Env, member: Address);
    
    /// (Admin only) Remove a member from yield distributions
    ///
    /// ### Arguments
    /// * `member` - The address of the member to remove
    ///
    /// ### Panics
    /// If the caller is not the admin
    /// If the member does not exist
    fn remove_member(e: &Env, member: Address);
    
    /// Fetch the list of active members eligible for yield distribution
    fn list_members(e: &Env) -> Vec<Address>;

    /// (Admin only) Set a new treasury address
    ///
    /// ### Arguments
    /// * `treasury` - The new treasury address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_treasury(e: &Env, treasury: Address);
    
    /// Fetch the treasury address
    fn get_treasury(e: &Env) -> Address;

    /// (Admin only) Set the treasury's share of distributions
    ///
    /// ### Arguments
    /// * `share_bps` - The treasury share in basis points (0-10000)
    ///
    /// ### Panics
    /// If the caller is not the admin
    /// If share_bps exceeds 10000
    fn set_treasury_share(e: &Env, share_bps: u32);
    
    /// Fetch the treasury's share in basis points
    fn get_treasury_share(e: &Env) -> u32;

    /// (Admin only) Set the distribution period
    ///
    /// ### Arguments
    /// * `period` - The new distribution period in seconds
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_distribution_period(e: &Env, period: u64);
    
    /// Fetch the distribution period in seconds
    fn get_distribution_period(e: &Env) -> u64;

    /// Fetch the current distribution information
    fn get_distribution_info(e: &Env) -> Distribution;
    
    /// Fetch the history of all distributions
    fn get_distribution_history(e: &Env) -> Vec<Distribution>;
    
    /// Fetch the timestamp of the next distribution
    fn get_next_distribution_time(e: &Env) -> u64;
    
    /// Check if a distribution is currently available
    fn is_distribution_available(e: &Env) -> bool;
    
    /// Fetch the time remaining until the next distribution (in seconds)
    fn time_before_next_distribution(e: &Env) -> u64;
    
    /// Fetch the current epoch number
    fn get_current_epoch(e: &Env) -> u64;

    /// (Yield Controller only) Distribute yield to treasury and members
    ///
    /// Returns the total amount distributed
    ///
    /// ### Arguments
    /// * `token` - The address of the token being distributed
    /// * `amount` - The total amount of yield to distribute
    ///
    /// ### Panics
    /// If the caller is not the yield controller
    /// If distribution is not yet available
    /// If there are no active members
    fn distribute_yield(e: &Env, token: Address, amount: i128) -> i128;
    
    /// Fetch the total amount of yield distributed across all epochs
    fn get_total_distributed(e: &Env) -> i128;
    
    /// (Owner only) Set a new admin address
    ///
    /// ### Arguments
    /// * `new_admin` - The new admin address
    ///
    /// ### Panics
    /// If the caller is not the owner
    fn set_admin(e: &Env, new_admin: Address);

    /// (Owner only) Upgrade the contract to a new WASM bytecode
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - The hash of the new WASM bytecode (must be uploaded first)
    ///
    /// ### Panics
    /// If the caller is not the owner
    fn upgrade(e: &Env, new_wasm_hash: BytesN<32>);
}

/// ### YieldDistributor
///
/// Manages the distribution of yield generated by the Coopstable protocol.
/// Distributes yield between the treasury and active members based on configured shares.
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
        storage::set_yield_controller(&e, &yield_controller);
        
        let config = storage_types::DistributionConfig {
            treasury_share_bps,
            distribution_period,
        };
        e.storage().instance().set(&storage_types::DataKey::DistributionConfig, &config);
        
        e.storage().instance().set(&storage_types::CURRENT_EPOCH_KEY, &0u64);
        
        e.storage().instance().extend_ttl(
            storage_types::INSTANCE_LIFETIME_THRESHOLD,
            storage_types::INSTANCE_BUMP_AMOUNT,
        );
        
        let initial_distribution = storage_types::Distribution {
            distribution_start_timestamp: e.ledger().timestamp(),
            epoch: 0,
            distribution_end_timestamp: 0,
            distribution_total: 0,
            distribution_treasury: 0,
            distribution_member: 0,
            members: Vec::new(&e),
            is_processed: false,
        };
        e.storage().persistent().set(&storage_types::DataKey::Distribution(0), &initial_distribution);
        e.storage().persistent().extend_ttl(
            &storage_types::DataKey::Distribution(0),
            storage_types::PERSISTENT_LIFETIME_THRESHOLD,
            storage_types::PERSISTENT_BUMP_AMOUNT,
        );   

        e.storage().persistent().set(&storage_types::DataKey::TotalDistributed, &0i128);
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

    fn get_treasury(e: &Env) -> Address { storage::get_treasury(e) }

    fn set_treasury_share(e: &Env, share_bps: u32) {
        require_admin(e);
        storage::set_treasury_share_bps(e, share_bps);
        YieldDistributorEvents::set_treasury_share(e, share_bps);
    }

    fn get_treasury_share(e: &Env) -> u32 { storage::get_treasury_share_bps(e) }

    fn set_distribution_period(e: &Env, period: u64) {
        require_admin(e);
        storage::set_distribution_period(e, period);
        YieldDistributorEvents::set_distribution_period(e, period);
    }

    fn get_distribution_period(e: &Env) -> u64 { storage::get_distribution_config(e).distribution_period }

    fn get_next_distribution_time(e: &Env) -> u64 { storage::read_next_distribution(e) }

    fn time_before_next_distribution(e: &Env) -> u64 { 
        let next_distribution = storage::read_next_distribution(e);
        let current_time = e.ledger().timestamp();
        
        if next_distribution <= current_time {
            return 0;
        }
        
        next_distribution - current_time
     }

    fn is_distribution_available(e: &Env) -> bool { storage::check_distribution_availability(e) }

    fn distribute_yield(e: &Env, token: Address, amount: i128) -> i128 {
        
        require_yield_controller(e);

        if !storage::check_distribution_availability(e) {
            return 0;
        }

        let distribution = storage::read_distribution_of_current_epoch(e);
        let treasury_share_bps = storage::get_treasury_share_bps(e);
        let treasury = storage::get_treasury(e);

        let mut treasury_amount = (amount as i128 * treasury_share_bps as i128) / 10000;
        let members_amount = amount - treasury_amount;
        
        let per_member_amount = if distribution.members.len() > 0 {
            members_amount / distribution.members.len() as i128
        } else {
            treasury_amount = amount; // if no members then it all goes to the treasury
            0
        };

        let token_client = TokenClient::new(e, &token);
        if per_member_amount > 0 {
            for member in distribution.members.iter() {
                utils::authenticate_contract(
                    &e, 
                    token_client.address.clone(), 
                    Symbol::new(&e, "transfer"), 
                    vec![
                        e,
                        (&e.current_contract_address()).into_val(e),
                        (&member).into_val(e),
                        (&per_member_amount).into_val(e),
                    ]
                );
                token_client.transfer(
                    &e.current_contract_address(),
                    &member,
                    &per_member_amount,
                );
            }
        }
        utils::authenticate_contract(
            &e, 
            token_client.address.clone(), 
            Symbol::new(&e, "transfer"), 
            vec![
                e,
                (&e.current_contract_address()).into_val(e),
                (&treasury).into_val(e),
                (&treasury_amount).into_val(e),
            ]
        );
        token_client.transfer(
            &e.current_contract_address(),
            &treasury,
            &treasury_amount,
        );
        
        storage::record_distribution(e, amount, treasury_amount, members_amount);
        
        YieldDistributorEvents::distribute_yield(
            e,
            token,
            amount,
            treasury_amount,
            distribution.members,
            per_member_amount,
        );

        amount
    }

    fn get_distribution_info(e: &Env) -> Distribution {  storage::read_distribution_of_current_epoch(e) }

    fn get_distribution_history(e: &Env) -> Vec<Distribution> { storage::read_distribution_history(e) }
    
    fn get_current_epoch(e: &Env) -> u64 { storage::read_epoch_current(e) }

    fn set_admin(e: &Env, new_admin: Address) {
        require_owner(e);
        storage::write_admin(e, new_admin.clone());
        YieldDistributorEvents::set_admin(&e, new_admin);
    }

    fn get_yield_controller(e: &Env) -> Address { storage::get_yield_controller(e) }

    fn get_total_distributed(e: &Env) -> i128 { storage::read_total_distributed(e) }

    fn upgrade(e: &Env, new_wasm_hash: BytesN<32>) {
        require_owner(e);
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
