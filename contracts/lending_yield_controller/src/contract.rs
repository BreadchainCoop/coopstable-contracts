use::soroban_sdk::{
    contract, 
    contractimpl, 
    Env,
    Address,
    Symbol,
    symbol_short,
    token::TokenClient
};
use crate::constants::{
    INSTANCE_BUMP_AMOUNT, 
    INSTANCE_LIFETIME_THRESHOLD
};
use yield_adapter::{
    lending_adapter::LendingAdapterClient,
    contract_types::SupportedAdapter
};
use cusd_manager::contract::CusdManagerClient;
use yield_adapter_registry::contract::YieldAdapterRegistryClient;

const ADAPTER_REGISTRY_KEY: Symbol = symbol_short!("AR");
const CUSD_MANAGER_KEY: Symbol = symbol_short!("CM");

pub trait LendingYieldControllerTrait {
    fn deposit_collateral(env: &Env, protocol: SupportedAdapter, user: Address, asset: Address, amount: i128) -> i128;
    fn withdraw_collateral(env: &Env, protocol: SupportedAdapter, user: Address, asset: Address, amount: i128) -> i128;
}

#[contract]
pub struct LendingYieldController ;

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

    fn cusd_manager_client(e: &Env) -> CusdManagerClient {
        CusdManagerClient::new(e, &Self::get_cusd_manager(&e))
    }

    fn adapter_registry_client(e: &Env) -> YieldAdapterRegistryClient {
        YieldAdapterRegistryClient::new(e, &Self::get_adapter_registry(&e))
    }
}

#[contractimpl]
impl LendingYieldControllerTrait for LendingYieldController {

    fn deposit_collateral(
        e: &Env,
        protocol: SupportedAdapter,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        user.require_auth();
        
        // deps
        let registry_client = Self::adapter_registry_client(&e);
        let cusd_manager_client = Self::cusd_manager_client(&e);
        let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&protocol));
        let asset_client = TokenClient::new(e, &asset);

        // require supported asset for adapter
        let is_asset_supported = registry_client.is_supported_asset(&protocol, &asset);
        if !is_asset_supported {
            panic!("Asset is not supported by the adapter registry");
        };
        
        // Deposit collateral
        adapter.deposit(&user, &asset, &amount);
        
        
        // Mint cUSD
        // note: yield controller should be given approval to transfer asset from user
        asset_client.transfer_from(
            &e.current_contract_address(), 
            &user, 
            &cusd_manager_client.address, 
            &amount
        );
        cusd_manager_client.issue_cusd(&user, &amount);
        
        amount
    }

    fn withdraw_collateral(
        e: &Env,
        protocol: SupportedAdapter,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        // Get the Lending Adapter
        let registry_client = Self::adapter_registry_client(&e);
        let cusd_manager_client = Self::cusd_manager_client(&e);
        let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&protocol));
        let asset_client = TokenClient::new(e, &asset);

        // require supported asset for adapter
        let is_asset_supported = registry_client.is_supported_asset(&protocol, &asset);
        if !is_asset_supported {
            panic!("Asset is not supported by the adapter registry");
        };
        
        // Withdraw collateral
        adapter.withdraw(&user, &asset, &amount);
        
        // Burn cUSD
        // will transfer usdc asset to yield controller
        cusd_manager_client.burn_cusd(&user, &amount);
                
        // Transfer asset to user
        asset_client.transfer(
            &e.current_contract_address(), 
            &user, 
            &amount
        );
        
        amount
    }
}


