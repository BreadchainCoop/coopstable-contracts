use soroban_sdk::Vec;
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
    contract_types::{SupportedAdapter,SupportedYieldType}
};
use cusd_manager::contract::CUSDManagerClient;
use yield_adapter_registry::contract::YieldAdapterRegistryClient;

const ADAPTER_REGISTRY_KEY: Symbol = symbol_short!("AR");
const CUSD_MANAGER_KEY: Symbol = symbol_short!("CM");
const YIELD_TYPE: SupportedYieldType = SupportedYieldType::Lending;
pub trait LendingYieldControllerTrait {
    fn __constructor(
        e: Env, 
        yield_distributor: Address,
        adapter_registry: Address, 
        cusd_manager: Address,
    );
    fn deposit_collateral(e: &Env, protocol: SupportedAdapter, user: Address, asset: Address, amount: i128) -> i128;
    fn withdraw_collateral(e: &Env, protocol: SupportedAdapter, user: Address, asset: Address, amount: i128) -> i128;
    fn get_yield(e: &Env) -> i128;
    fn claim_yield(e: &Env) -> i128;
}

#[contract]
pub struct LendingYieldController ;

impl LendingYieldController {
    
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

    fn cusd_manager_client(e: &Env) -> CUSDManagerClient {
        CUSDManagerClient::new(e, &Self::get_cusd_manager(&e))
    }

    fn adapter_registry_client(e: &Env) -> YieldAdapterRegistryClient {
        YieldAdapterRegistryClient::new(e, &Self::get_adapter_registry(&e))
    }
}

#[contractimpl]
impl LendingYieldControllerTrait for LendingYieldController {
    fn __constructor(
        e: Env,
        yield_distributor: Address,
        adapter_registry: Address, 
        cusd_manager: Address,
    ) {
        e.storage().instance().extend_ttl(
            INSTANCE_LIFETIME_THRESHOLD, 
            INSTANCE_BUMP_AMOUNT
        );
        e.storage().instance().set(&ADAPTER_REGISTRY_KEY, &adapter_registry);
        e.storage().instance().set(&CUSD_MANAGER_KEY, &cusd_manager);
    }
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
        let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&YIELD_TYPE, &protocol));
        let asset_client = TokenClient::new(e, &asset);

        // require supported asset for adapter
        let is_asset_supported = registry_client.is_supported_asset(&YIELD_TYPE,&protocol, &asset);
        if !is_asset_supported {
            panic!("Asset is not supported by the adapter registry");
        };
        
        // Deposit collateral
        asset_client.transfer_from(
            &e.current_contract_address(), 
            &user, 
            &e.current_contract_address(), 
            &amount
        );
        asset_client.approve(&e.current_contract_address(), &adapter.address, &amount, &u32::MAX); 
        adapter.deposit(&e.current_contract_address(), &asset, &amount);
        
        // note: yield controller should be given approval to transfer asset from user
        cusd_manager_client.issue_cusd(&e.current_contract_address(), &user, &amount);
        
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
        let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&YIELD_TYPE,&protocol));
        let asset_client = TokenClient::new(e, &asset);

        // require supported asset for adapter
        let is_asset_supported = registry_client.is_supported_asset(&YIELD_TYPE, &protocol, &asset);
        if !is_asset_supported {
            panic!("Asset is not supported by the adapter registry");
        };
        
        // Withdraw collateral
        adapter.withdraw(&user, &asset, &amount);
        
        // Burn cUSD
        // will transfer usdc asset to yield controller
        cusd_manager_client.burn_cusd(&e.current_contract_address(),&user, &amount);
                
        // Transfer asset to user
        asset_client.transfer(
            &e.current_contract_address(), 
            &user, 
            &amount
        );
        
        amount
    }

    fn get_yield(e: &Env) -> i128 {
        let registry_client = Self::adapter_registry_client(e);
        let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&YIELD_TYPE);
        let user = e.current_contract_address();
        
        // Use fold to accumulate the total yield across all adapters and assets
        lend_protocols_with_assets.iter().fold(0, |adapter_acc, (adapter_address, supported_assets)| {
            let adapter_client = LendingAdapterClient::new(e, &adapter_address);
            
            let adapter_total = supported_assets.iter().fold(0, |asset_acc, asset| {
                let asset_yield = adapter_client.get_yield(&user, &asset);
                asset_acc + asset_yield
            });
            adapter_acc + adapter_total
        })
    }
    
    fn claim_yield(e: &Env) -> i128 {
        0
    }
}

