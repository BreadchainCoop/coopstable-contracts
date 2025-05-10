use soroban_sdk::{Address, Env};
use crate::constants::{
    ADAPTER_REGISTRY_KEY, 
    CUSD_MANAGER_KEY, 
    INSTANCE_BUMP_AMOUNT, 
    INSTANCE_LIFETIME_THRESHOLD,
    YIELD_DISTRIBUTOR_KEY,
};
use crate::cusd_manager::Client as CUSDManagerClient;
use crate::yield_adapter_registry::Client as YieldAdapterRegistryClient;
use crate::yield_distributor::Client as YieldDistributorClient;

pub fn get_cusd_manager(e: &Env) -> Address {
    e.storage()
        .instance()
        .extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT);
    e.storage().instance().get(&CUSD_MANAGER_KEY).unwrap()
}

pub fn get_adapter_registry(e: &Env) -> Address {
    e.storage()
        .instance()
        .extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT);
    e.storage().instance().get(&ADAPTER_REGISTRY_KEY).unwrap()
}

pub fn get_yield_distributor(e: &Env) -> Address {
    e.storage()
        .instance()
        .extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
    e.storage().instance().get(&YIELD_DISTRIBUTOR_KEY).unwrap()
}

pub fn set_yield_distributor(e: &Env, yield_distributor: Address) {
    e.storage()
        .instance()
        .extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT);
    e.storage().instance().set(
        &YIELD_DISTRIBUTOR_KEY, 
        &yield_distributor
    );
}

pub fn set_cusd_manager(e: &Env, cusd_manager: Address) {
    e.storage()
        .instance()
        .extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
    e.storage().instance().set(
        &CUSD_MANAGER_KEY, 
        &cusd_manager
    );
}

pub fn set_adapter_registry(e: &Env, adapter_registry: Address) {
    e.storage()
        .instance()
        .extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
    e.storage().instance().set(
        &ADAPTER_REGISTRY_KEY, 
        &adapter_registry
    );
}

pub fn distributor_client(e: &Env) -> YieldDistributorClient {
    YieldDistributorClient::new(e, &get_yield_distributor(&e))
}

pub fn cusd_manager_client(e: &Env) -> CUSDManagerClient {
    CUSDManagerClient::new(
        e, 
        &get_cusd_manager(&e)
    )
}

pub fn adapter_registry_client(e: &Env) -> YieldAdapterRegistryClient<'static> {
    YieldAdapterRegistryClient::new(
        e, 
        &get_adapter_registry(&e)
    )
}