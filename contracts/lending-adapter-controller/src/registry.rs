use soroban_sdk::{
    contract, 
    contracterror, 
    contractimpl, 
    contractmeta, 
    contracttype, 
    panic_with_error, 
    symbol_short, 
    vec,
    Symbol,
    Address, 
    Env, 
    Map, 
    Vec
};
use crate::storage_types::LendingAdapterRegistry;
use crate::registry_storage::{
    register_lending_adapter, 
    remove_lending_adapter, 
    verify_if_lending_adapter_exists
};

contractmeta!(
    key = "Description",
    val = "Lending Pool Registry for the Coopstable cUSD system"
);

trait LendingAdapterRegistryTrait {
    fn register_adapter(env: &Env, protocol: Symbol, address: Address);
    fn remove_lending_protocol(env: &Env, protocol: Symbol, address: Address);
    fn get_lending_protocol(env: &Env, protocol: Symbol) -> Address;
}

pub struct LendingAdapterRegistry;

#[contractimpl]
impl LendingAdapterRegistryTrait for LendingProtocolRegistry {
    fn register_adapter(
        env: &Env, 
        protocol: Symbol, 
        address: Address
    ) {
        register_lending_adapter(env, protocol, address);
    }

    fn remove_lending_protocol(env: &Env, protocol: Symbol, address: Address) {
        
    }

    fn get_lending_protocol(env: &Env, protocol: Symbol) -> Address {
        
    }
}
