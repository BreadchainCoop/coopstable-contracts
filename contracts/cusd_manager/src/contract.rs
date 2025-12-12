use crate::events::CUSDManagerEvents;
use crate::error::CUSDManagerError;
use crate::storage_types::DataKey;
use crate::token;
use crate::storage;
use soroban_sdk::{ contract, contractimpl, contractmeta, Address, BytesN, Env, panic_with_error };

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

/// ### CUSDManager
///
/// Manager contract responsible for the cUSD stablecoin operations including minting,
/// burning, and supply tracking. Acts as the central authority for cUSD token management.
#[contract]
pub struct CUSDManager;

pub trait CUSDManagerTrait {
    /// Initialize the cUSD Manager contract
    ///
    /// ### Arguments
    /// * `cusd_id` - The address of the cUSD token contract
    /// * `owner` - The address of the contract owner (can set admin)
    /// * `admin` - The address of the admin (manages operational settings)
    fn __constructor(e: Env, cusd_id: Address, owner: Address, admin: Address);
    
    /// (Yield Controller only) Issue new cUSD tokens to a specified address
    ///
    /// ### Arguments
    /// * `to` - The recipient address for the newly minted cUSD tokens
    /// * `amount` - The amount of cUSD tokens to mint
    ///
    /// ### Panics
    /// If the caller is not the authorized yield controller
    /// If the amount is negative
    fn issue_cusd(e: &Env, to: Address, amount: i128);
    
    /// Burn cUSD tokens from a specified address
    ///
    /// ### Arguments
    /// * `from` - The address from which to burn cUSD tokens
    /// * `amount` - The amount of cUSD tokens to burn
    ///
    /// ### Panics
    /// If the amount is negative
    /// If the address has insufficient balance
    fn burn_cusd(e: &Env, from: Address, amount: i128);
    
    /// Fetch the address of the cUSD token contract
    fn get_cusd_id(e: &Env) -> Address;
    
    /// Fetch the total supply of cUSD tokens in circulation
    fn cusd_total_supply(e: &Env) -> i128;
    
    /// (Owner only) Set a new admin address
    ///
    /// ### Arguments
    /// * `new_admin` - The new admin address
    ///
    /// ### Panics
    /// If the caller is not the owner
    fn set_admin(e: &Env, new_admin: Address); 
    
    /// (Admin only) Set a new yield controller address
    ///
    /// ### Arguments
    /// * `new_controller` - The new yield controller address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_yield_controller(e: &Env, new_controller: Address);
    
    /// (Admin only) Set a new cUSD token contract address
    ///
    /// ### Arguments
    /// * `new_cusd_id` - The new cUSD token contract address
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_cusd_id(e: &Env, new_cusd_id: Address);
    
    /// (Admin only) Set a new issuer for the cUSD token contract
    ///
    /// ### Arguments
    /// * `new_issuer` - The new issuer address for the cUSD token
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn set_cusd_issuer(e: &Env, new_issuer: Address);

    /// (Owner only) Upgrade the contract to a new WASM bytecode
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - The hash of the new WASM bytecode (must be uploaded first)
    ///
    /// ### Panics
    /// If the caller is not the owner
    fn upgrade(e: &Env, new_wasm_hash: BytesN<32>);
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

    fn upgrade(e: &Env, new_wasm_hash: BytesN<32>) {
        require_owner(e);
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}
