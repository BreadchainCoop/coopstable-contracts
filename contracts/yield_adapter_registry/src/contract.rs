use crate::{
    events::YieldAdapterRegistryEvents,
    error::YieldAdapterRegistryError,
    storage
};
use soroban_sdk::{contract, contractimpl, contractmeta, panic_with_error, Address, Env, Symbol, Vec};

contractmeta!(
    key = "Description",
    val = "Yield adapter registry for the Coopstable cUSD system"
);

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }

pub trait YieldAdapterRegistryTrait {
    /// Initialize the Yield Adapter Registry contract
    ///
    /// ### Arguments
    /// * `admin` - The address of the admin (manages adapter registration)
    /// * `owner` - The address of the contract owner (can set admin)
    fn __constructor(e: Env, admin: Address, owner: Address);
    
    /// (Owner only) Set a new admin address for the registry
    ///
    /// ### Arguments
    /// * `new_admin` - The new admin address
    ///
    /// ### Panics
    /// If the caller is not the owner
    fn set_yield_adapter_admin(e: &Env, new_admin: Address);
    
    /// (Admin only) Register a new yield adapter for a specific protocol
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation (e.g., 'lending')
    /// * `protocol` - The protocol identifier (e.g., 'blend')
    /// * `adapter_address` - The address of the adapter contract
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn register_adapter(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        adapter_address: Address,
    );
    
    /// Fetch the adapter address for a specific yield type and protocol
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    /// * `protocol` - The protocol identifier
    ///
    /// ### Panics
    /// If no adapter is registered for the given yield type and protocol
    fn get_adapter(e: &Env, yield_type: Symbol, protocol: Symbol) -> Address;
    
    /// Fetch all adapter addresses for a specific yield type
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    fn get_adapters(e: &Env, yield_type: Symbol) -> Vec<Address>;
    
    /// Fetch all adapters with their supported assets for a specific yield type
    ///
    /// Returns a vector of tuples containing adapter addresses and their supported assets
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    fn get_adapters_with_assets(e: &Env, yield_type: Symbol) -> Vec<(Address, Vec<Address>)>;
    
    /// (Admin only) Remove a registered adapter
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    /// * `protocol` - The protocol identifier
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn remove_adapter(e: &Env, yield_type: Symbol, protocol: Symbol);
    
    /// (Admin only) Add support for a specific asset in an adapter
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    /// * `protocol` - The protocol identifier
    /// * `asset_address` - The address of the asset to support
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn add_support_for_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    );
    
    /// (Admin only) Remove support for a specific asset in an adapter
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    /// * `protocol` - The protocol identifier
    /// * `asset_address` - The address of the asset to remove support for
    ///
    /// ### Panics
    /// If the caller is not the admin
    fn remove_support_for_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    );
    
    /// Check if an asset is supported by a specific adapter
    ///
    /// ### Arguments
    /// * `yield_type` - The type of yield generation
    /// * `protocol` - The protocol identifier
    /// * `asset_address` - The address of the asset to check
    fn is_supported_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    ) -> bool;
}

/// ### YieldAdapterRegistry
///
/// Registry contract that manages yield adapters for different protocols.
/// Maintains a mapping of yield types and protocols to their corresponding adapter contracts.
#[contract]
pub struct YieldAdapterRegistry;

#[contractimpl]
impl YieldAdapterRegistryTrait for YieldAdapterRegistry {
    fn __constructor(e: Env, admin: Address, owner: Address) {
        storage::write_owner(&e, owner);
        storage::write_admin(&e, admin);
    }

    fn set_yield_adapter_admin(e: &Env, new_admin: Address) {
        require_owner(e);
        YieldAdapterRegistryEvents::set_admin(&e, new_admin);
    }

    fn register_adapter(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        adapter_address: Address,
    ) {
        require_admin(e);
        storage::register_yield_adapter(
            e,
            yield_type.clone(),
            protocol.clone(),
            adapter_address.clone(),
        );
        YieldAdapterRegistryEvents::register_adapter(&e, yield_type, protocol, adapter_address);
    }

    fn remove_adapter(e: &Env, yield_type: Symbol, protocol: Symbol) {
        require_admin(e);
        let adapter_address = storage::get_yield_adapter(e, yield_type.clone(), protocol.clone());
        storage::remove_yield_adapter(e, yield_type.clone(), protocol.clone());
        YieldAdapterRegistryEvents::remove_adapter(&e, yield_type, protocol, adapter_address);
    }

    fn get_adapter(e: &Env, yield_type: Symbol, protocol: Symbol) -> Address {
        if !storage::verify_if_yield_adapter_exists(e, yield_type.clone(), protocol.clone()) {
            panic_with_error!(e, YieldAdapterRegistryError::InvalidYieldAdapter);
        }
        storage::get_yield_adapter(e, yield_type, protocol)
    }

    fn add_support_for_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    ) {
        require_admin(e);
        storage::support_asset(
            e,
            yield_type.clone(),
            protocol.clone(),
            asset_address.clone(),
        );
        YieldAdapterRegistryEvents::add_support_for_asset(&e, yield_type, protocol, asset_address);
    }

    fn remove_support_for_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    ) {
        require_admin(e);
        storage::remove_asset_support(
            &e,
            yield_type.clone(),
            protocol.clone(),
            asset_address.clone(),
        );
        YieldAdapterRegistryEvents::remove_support_for_asset(
            &e,
            yield_type,
            protocol,
            asset_address,
        );
    }

    fn is_supported_asset(
        e: &Env,
        yield_type: Symbol,
        protocol: Symbol,
        asset_address: Address,
    ) -> bool {
        storage::is_asset_supported(e, yield_type, protocol, asset_address)
    }

    fn get_adapters(e: &Env, yield_type: Symbol) -> Vec<Address> {
        storage::get_yield_adapters(e, yield_type)
    }

    fn get_adapters_with_assets(e: &Env, yield_type: Symbol) -> Vec<(Address, Vec<Address>)> {
        storage::get_yield_adapters_with_assets(e, yield_type)
    }
}
