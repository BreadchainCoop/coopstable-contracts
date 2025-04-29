use::soroban_sdk::{
    contract, 
    contractimpl, 
    Env,
    Address,
    Symbol,
    symbol_short
};
use crate::constants::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use yield_adapter::lending_adapter::LendingAdapterClient;
use cusd_manager::contract::CusdManagerClient;
use yield_adapter_registry::contract::YieldAdapterRegistryClient;

const ADAPTER_REGISTRY_KEY: Symbol = symbol_short!("AR");
const CUSD_MANAGER_KEY: Symbol = symbol_short!("CM");

pub trait LendingYieldControllerTrait {
    fn deposit_collateral(env: &Env, user: Address, asset: Address, amount: i128);
    fn withdraw_collateral(env: &Env, user: Address, asset: Address, amount: i128);
}

#[contract]
pub struct LendingYieldController;

#[contractimpl]
impl LendingYieldController {
    
    fn __constructor(
        env: Env, 
        adapter_registry: Address, 
        cusd_manager: Address
    ) {
        env.storage().instance().extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
        env.storage().instance().set(&ADAPTER_REGISTRY_KEY, &adapter_registry);
        env.storage().instance().set(&CUSD_MANAGER_KEY, &cusd_manager);
    }
    fn get_cusd_manager(e: &Env) -> Address {
        e.storage().instance().extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
        e.storage().instance().get(&CUSD_MANAGER_KEY).unwrap()
    }

    fn get_adapter_registry(e: &Env) -> Address {
        e.storage().instance().extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
        e.storage().instance().get(&ADAPTER_REGISTRY_KEY).unwrap()
    }

    fn get_cusd_manager_client(e: &Env) -> CusdManagerClient {
        CusdManagerClient::new(e, &Self::get_cusd_manager(&e))
    }

    fn get_adapter_registry_client(e: &Env) -> YieldAdapterRegistryClient {
        YieldAdapterRegistryClient::new(e, &Self::get_adapter_registry(&e))
    }
}

#[contractimpl]
impl LendingYieldControllerTrait for LendingYieldController {


    fn deposit_collateral(
        e: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        // deps
        let registry_client = Self::get_adapter_registry_client(&e);
        let cusd_manager_client = Self::get_cusd_manager_client(&e);
        let adapter = LendingAdapterClient::new(&e, &registry_client.get_adapter(&e, asset));
        
        // Get the Lending Adapter
        
        
        // Deposit collateral
        adapter.deposit(&user, &asset, &amount);
        
        // Mint cUSD
        e.token().mint(&cusd_manager, &user, amount);
        
        amount
    }
}


