use soroban_sdk::{Address, Env, Symbol};
use crate::storage_types::{
    DataKey,
    PendingHarvest,
    INSTANCE_BUMP_AMOUNT,
    INSTANCE_LIFETIME_THRESHOLD,
};
use crate::cusd_manager::Client as CUSDManagerClient;
use crate::yield_adapter_registry::Client as YieldAdapterRegistryClient;
use crate::yield_distributor::Client as YieldDistributorClient;

pub fn adapter_registry_client(e: &Env) -> YieldAdapterRegistryClient<'static> {
    YieldAdapterRegistryClient::new(
        e, 
        &get_adapter_registry(&e)
    )
}
pub fn extend_instance(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}
fn read_address(e: &Env, key: &DataKey) -> Address {
    extend_instance(e);
    e.storage().instance().get(key).unwrap()  
}
fn write_address(e: &Env, key: &DataKey, address: &Address) {
    extend_instance(e);    
    e.storage().instance().set(key, address); 
}
pub fn read_admin(e: &Env) -> Address { read_address(e, &DataKey::Admin)}
pub fn read_owner(e: &Env) -> Address { read_address(e, &DataKey::Owner)}
pub fn write_admin(e: &Env, new_admin: Address) { write_address(e, &DataKey::Admin, &new_admin);}
pub fn write_owner(e: &Env, new_owner: Address) { write_address(e, &DataKey::Owner, &new_owner);}
pub fn get_cusd_manager(e: &Env) -> Address { read_address(e, &DataKey::CUSDManager) }
pub fn get_adapter_registry(e: &Env) -> Address { read_address(e, &DataKey::AdapterRegistry) }
pub fn get_yield_distributor(e: &Env) -> Address { read_address(e, &DataKey::YieldDistributor) }
pub fn set_yield_distributor(e: &Env, yield_distributor: Address) {
    write_address(e, &DataKey::YieldDistributor, &yield_distributor);
}
pub fn set_cusd_manager(e: &Env, cusd_manager: Address) {
    write_address(e, &DataKey::CUSDManager, &cusd_manager);
}
pub fn set_adapter_registry(e: &Env, adapter_registry: Address) { write_address(e, &DataKey::AdapterRegistry, &adapter_registry); }
pub fn distributor_client(e: &Env) -> YieldDistributorClient {
    YieldDistributorClient::new(e, &get_yield_distributor(&e))
}
pub fn cusd_manager_client(e: &Env) -> CUSDManagerClient {
    CUSDManagerClient::new(
        e,
        &get_cusd_manager(&e)
    )
}

// Pending harvest storage functions
pub fn get_pending_harvest(e: &Env, protocol: &Symbol, asset: &Address) -> Option<PendingHarvest> {
    extend_instance(e);
    e.storage().instance().get(&DataKey::PendingHarvest(protocol.clone(), asset.clone()))
}

pub fn set_pending_harvest(e: &Env, harvest: &PendingHarvest) {
    extend_instance(e);
    e.storage().instance().set(
        &DataKey::PendingHarvest(harvest.protocol.clone(), harvest.asset.clone()),
        harvest
    );
}

pub fn remove_pending_harvest(e: &Env, protocol: &Symbol, asset: &Address) {
    extend_instance(e);
    e.storage().instance().remove(&DataKey::PendingHarvest(protocol.clone(), asset.clone()));
}

pub fn has_pending_harvest(e: &Env, protocol: &Symbol, asset: &Address) -> bool {
    extend_instance(e);
    e.storage().instance().has(&DataKey::PendingHarvest(protocol.clone(), asset.clone()))
}
