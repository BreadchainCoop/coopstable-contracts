use soroban_sdk::{ contract, contractimpl, contractmeta, panic_with_error, Address, Env, Symbol};
use crate::error::LendingYieldControllerError;
use crate::events::LendingYieldControllerEvents;
use crate::{storage, controls};

contractmeta!(
    key = "Description",
    val = "Yield controller for the Coopstable cUSD system"
);

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }

pub trait LendingYieldControllerTrait {
    /// Initialize the Lending Yield Controller contract
    ///
    /// ### Arguments
    /// * `yield_distributor` - The address of the yield distributor contract
    /// * `adapter_registry` - The address of the adapter registry contract
    /// * `cusd_manager` - The address of the cUSD manager contract
    /// * `admin` - The address of the admin (manages operational settings)
    /// * `owner` - The address of the contract owner (can set admin)
    fn __constructor(
        e: Env,
        yield_distributor: Address,
        adapter_registry: Address,
        cusd_manager: Address,
        admin: Address,
        owner: Address,
    );
    
    /// (Admin only) Set a new yield distributor contract address
    ///
    /// ### Arguments
    /// * `yield_distributor` - The new yield distributor contract address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_yield_distributor(e: &Env, yield_distributor: Address);
    
    /// Fetch the address of the yield distributor contract
    fn get_yield_distributor(e: &Env) -> Address;
    
    /// (Admin only) Set a new adapter registry contract address
    ///
    /// ### Arguments
    /// * `adapter_registry` - The new adapter registry contract address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_adapter_registry(e: &Env, adapter_registry: Address);
    
    /// Fetch the address of the adapter registry contract
    fn get_adapter_registry(e: &Env) -> Address;
    
    /// (Admin only) Set a new cUSD manager contract address
    ///
    /// ### Arguments
    /// * `cusd_manager` - The new cUSD manager contract address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_cusd_manager(e: &Env, cusd_manager: Address);
    
    /// Fetch the address of the cUSD manager contract
    fn get_cusd_manager(e: &Env) -> Address;
    
    /// (Owner only) Set a new admin address
    ///
    /// ### Arguments
    /// * `new_admin` - The new admin address
    ///
    /// ### Panics
    /// If the caller is not the owner
    fn set_admin(e: &Env, new_admin: Address);
    
    /// Deposit collateral into a lending protocol through the yield controller
    ///
    /// Returns the actual amount deposited into the protocol
    ///
    /// ### Arguments
    /// * `protocol` - The symbol identifier of the lending protocol
    /// * `user` - The address of the user depositing collateral
    /// * `asset` - The address of the asset being deposited
    /// * `amount` - The amount of the asset to deposit
    ///
    /// ### Panics
    /// If the user does not authorize the transaction
    /// If the protocol is not registered in the adapter registry
    fn deposit_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128;
    
    /// Withdraw collateral from a lending protocol through the yield controller
    ///
    /// Returns the actual amount withdrawn from the protocol
    ///
    /// ### Arguments
    /// * `protocol` - The symbol identifier of the lending protocol
    /// * `user` - The address of the user withdrawing collateral
    /// * `asset` - The address of the asset being withdrawn
    /// * `amount` - The amount of the asset to withdraw
    ///
    /// ### Panics
    /// If the user does not authorize the transaction
    /// If the user has insufficient deposited balance
    fn withdraw_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128;
    
    /// Fetch the accumulated yield available for distribution
    fn get_yield(e: &Env) -> i128;
    
    /// Claim accumulated yield and distribute it through the yield distributor
    ///
    /// Returns the total amount of yield claimed and distributed
    ///
    /// ### Panics
    /// If no yield is available to claim
    /// If distribution is not available from the yield distributor
    fn claim_yield(e: &Env) -> i128;
    
    /// Claim emissions rewards from a specific protocol for an asset
    ///
    /// Returns the total amount of emissions claimed
    ///
    /// ### Arguments
    /// * `protocol` - The symbol identifier of the lending protocol
    /// * `asset` - The address of the asset for which to claim emissions
    fn claim_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128;
    
    /// Fetch the accumulated emissions rewards for a specific protocol and asset
    ///
    /// ### Arguments
    /// * `protocol` - The symbol identifier of the lending protocol
    /// * `asset` - The address of the asset to check emissions for
    fn get_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128;
    
    /// Fetch the total APY across all protocols for a specific asset
    ///
    /// ### Arguments
    /// * `asset` - The address of the asset to check APY for
    fn get_total_apy(e: &Env, asset: Address) -> u32;
    
    /// Fetch the weighted average APY across all assets and protocols
    fn get_weighted_total_apy(e: &Env) -> u32;
}


/// ### LendingYieldController
///
/// Central controller for managing yield generation across multiple lending protocols.
/// Handles deposits, withdrawals, yield aggregation, and distribution to the Coopstable ecosystem.
#[contract]
pub struct LendingYieldController;

#[contractimpl]
impl LendingYieldControllerTrait for LendingYieldController {
    fn __constructor(
        e: Env,
        yield_distributor: Address,
        adapter_registry: Address,
        cusd_manager: Address,
        admin: Address,
        owner: Address,
    ) {
        storage::set_yield_distributor(&e, yield_distributor);
        storage::set_adapter_registry(&e, adapter_registry);
        storage::set_cusd_manager(&e, cusd_manager);
        storage::write_admin(&e, admin); 
        storage::write_owner(&e, owner);
    }

    fn deposit_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128 {
        user.require_auth();
        
        let deposited = controls::process_deposit(&e, &protocol, user.clone(), asset.clone(), amount);
        
        LendingYieldControllerEvents::deposit_collateral(&e, user, asset, amount);

        deposited
    }

    fn withdraw_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128 {
        user.require_auth();
        
        let withdrawn = controls::process_withdraw_collateral(&e, &protocol, user.clone(), asset.clone(), amount);
        
        LendingYieldControllerEvents::withdraw_collateral(&e, user, asset, amount);
        
        withdrawn
    }

    fn get_yield(e: &Env) -> i128 { controls::read_yield(e) }

    fn claim_yield(e: &Env) -> i128 {
        
        if controls::read_yield(e) <= 0 {
            return 0;
        }
        
        let distributor = storage::distributor_client(e);
        if !distributor.is_distribution_available() {
            panic_with_error!(e, LendingYieldControllerError::YieldUnavailable);
        }

        let claimed_total = controls::process_claim_and_distribute_yield(e);
        
        let cusd_manager = storage::cusd_manager_client(e);

        LendingYieldControllerEvents::claim_yield(
            &e,
            e.current_contract_address(),
            cusd_manager.get_cusd_id(),
            claimed_total,
        );

        claimed_total
    }

    fn get_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128 {
        controls::read_emissions(e, &protocol, asset.clone())
    }

    fn claim_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128 {
        
        let claimed_total = controls::process_claim_emissions(e, &protocol, asset.clone());

        LendingYieldControllerEvents::claim_emissions(
            &e,
            e.current_contract_address(),
            asset,
            claimed_total,
        );

        claimed_total
    }

    fn set_yield_distributor(e: &Env, yield_distributor: Address) {
        require_admin(e);
        storage::set_yield_distributor(e, yield_distributor.clone());
        LendingYieldControllerEvents::set_yield_distributor(e, yield_distributor.clone());
    }

    fn set_adapter_registry(e: &Env, adapter_registry: Address) {
        require_admin(e);
        storage::set_adapter_registry(e, adapter_registry.clone());
        LendingYieldControllerEvents::set_adapter_registry(e, adapter_registry.clone());
    }
    
    fn set_cusd_manager(e: &Env, cusd_manager: Address) {
        require_admin(e);
        storage::set_cusd_manager(e, cusd_manager.clone());
        LendingYieldControllerEvents::set_cusd_manager(e, cusd_manager.clone());
    }

    fn get_yield_distributor(e: &Env) -> Address {
        storage::get_yield_distributor(e)
    }

    fn get_adapter_registry(e: &Env) -> Address {
        storage::get_adapter_registry(e)
    }

    fn get_cusd_manager(e: &Env) -> Address {
        storage::get_cusd_manager(e)
    }

    fn set_admin(e: &Env, new_admin: Address) {
        require_owner(e);
        storage::write_admin(e, new_admin.clone());
        LendingYieldControllerEvents::set_admin(e, new_admin);
    }
    
    fn get_total_apy(e: &Env, asset: Address) -> u32 {
        controls::calculate_asset_weighted_apy(e, asset)
    }
    
    fn get_weighted_total_apy(e: &Env) -> u32 {
        controls::calculate_portfolio_weighted_apy(e)
    }
}
