use soroban_sdk::{panic_with_error, contractmeta};
use crate::{
    storage_types, 
    error::LendingYieldControllerError, 
    events::LendingYieldControllerEvents,
};
use crate::storage;
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};
use yield_adapter::lending_adapter::LendingAdapterClient;

contractmeta!(
    key = "Description",
    val = "Yield controller for the Coopstable cUSD system"
);

fn require_admin(e: &Env) { storage::read_admin(e).require_auth(); }
fn require_owner(e: &Env) { storage::read_owner(e).require_auth(); }

pub trait LendingYieldControllerTrait {
    fn __constructor(
        e: Env,
        yield_distributor: Address,
        adapter_registry: Address,
        cusd_manager: Address,
        admin: Address,
        owner: Address,
    );
    fn set_yield_distributor(e: &Env, yield_distributor: Address);
    fn get_yield_distributor(e: &Env) -> Address;
    fn set_adapter_registry(e: &Env, adapter_registry: Address);
    fn get_adapter_registry(e: &Env) -> Address;
    fn set_cusd_manager(e: &Env, cusd_manager: Address);
    fn get_cusd_manager(e: &Env) -> Address;
    fn set_admin(e: &Env, new_admin: Address);

    fn deposit_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128;
    fn withdraw_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128;
    fn get_yield(e: &Env) -> i128;
    fn claim_yield(e: &Env) -> i128;
    fn claim_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128;
}

fn read_yield(e: &Env) -> i128 {
    let registry_client = storage::adapter_registry_client(e);
    let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&storage_types::YIELD_TYPE.id());
   
    lend_protocols_with_assets.iter().fold(
        0,
        |adapter_acc, (adapter_address, supported_assets)| {
            let adapter_client = LendingAdapterClient::new(e, &adapter_address);

            let adapter_total = supported_assets.iter().fold(0, |asset_acc, asset| {
                let asset_yield = adapter_client.get_yield(&asset);
                asset_acc + asset_yield
            });
            adapter_acc + adapter_total
        },
    )
}

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
        
        let registry_client = storage::adapter_registry_client(&e);
        let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
        
        adapter.deposit(
            &user, 
            &asset, 
            &amount
        );
        
        let cusd_manager_client = storage::cusd_manager_client(&e);
        // TODO - add require auth for controller
        cusd_manager_client.issue_cusd(&user, &amount);
        LendingYieldControllerEvents::deposit_collateral(&e, user, asset, amount);
        amount
    }

    fn withdraw_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128 {
        user.require_auth();

        let registry_client = storage::adapter_registry_client(&e);
        let cusd_manager_client = storage::cusd_manager_client(&e);
        let adapter =
            LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
        let is_asset_supported =
            registry_client.is_supported_asset(&storage_types::YIELD_TYPE.id(), &protocol, &asset);
        if !is_asset_supported {
            panic_with_error!(e, LendingYieldControllerError::UnsupportedAsset);
        };
        adapter.withdraw(&user, &asset, &amount);
        cusd_manager_client.burn_cusd(
            &user,
            &amount,
        );        
        LendingYieldControllerEvents::withdraw_collateral(&e, user, asset, amount);
        amount
    }

    fn get_yield(e: &Env) -> i128 { read_yield(e) }

    fn claim_yield(e: &Env) -> i128 {
        let total_yield = read_yield(e);

        if total_yield <= 0 {
            return 0;
        }

        let distributor = storage::distributor_client(e);
        if !distributor.is_distribution_available() {
            panic_with_error!(e, LendingYieldControllerError::YieldUnavailable);
        }

        let mut total_claimed: i128 = 0;
        let registry_client = storage::adapter_registry_client(e);
        let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&storage_types::YIELD_TYPE.id());
        for (adapter_address, supported_assets) in lend_protocols_with_assets.iter() {
            let adapter_client = LendingAdapterClient::new(e, &adapter_address);

            for asset in supported_assets.iter() {

                let claimed = adapter_client.claim_yield(&asset, &distributor.address);
                
                if claimed > 0 {
                    // TODO - add require auth for controller
                    distributor.distribute_yield(&asset, &claimed);
                    LendingYieldControllerEvents::claim_yield(
                        &e,
                        e.current_contract_address(),
                        asset.clone(),
                        claimed,
                    );

                    total_claimed += claimed;
                }

            }
        }

        total_claimed
    }

    fn claim_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128 {
        let registry_client = storage::adapter_registry_client(e);
        let adapter_client = LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
        let emissions = adapter_client.get_emissions(&e.current_contract_address(), &asset);
        if emissions > 0 {
            let distributor = storage::distributor_client(e);
            
            // TODO - add require auth for controller
            let claimed = adapter_client.claim_emissions(&distributor.get_treasury(), &asset);
            
            LendingYieldControllerEvents::claim_emissions(
                &e,
                distributor.get_treasury(),
                asset.clone(),
                claimed,
            );            
        }
        0
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
}
