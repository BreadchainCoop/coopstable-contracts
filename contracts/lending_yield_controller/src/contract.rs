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
    fn get_emissions(e: &Env, protocol: Symbol, asset: Address) -> i128;
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
}
