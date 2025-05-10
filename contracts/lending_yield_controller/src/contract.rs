use crate::{
    constants::{
        ADAPTER_REGISTRY_KEY, CUSD_MANAGER_KEY, INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
        YIELD_DISTRIBUTOR_KEY, YIELD_TYPE, YIELD_CONTROLLER_ADMIN_ROLE
    },
    events::LendingYieldControllerEvents,
};
use access_control::{
    access::default_access_control,
    constants::DEFAULT_ADMIN_ROLE,
};
use ::soroban_sdk::{contract, contractimpl, token::TokenClient, Address, Env, Symbol};
use soroban_sdk::log;
use yield_adapter::lending_adapter::LendingAdapterClient;
use crate::storage::{
    adapter_registry_client, 
    cusd_manager_client, 
    distributor_client, 
    get_adapter_registry, 
    get_cusd_manager, 
    get_yield_distributor, 
    set_adapter_registry, 
    set_cusd_manager, 
    set_yield_distributor
};

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
    fn set_admin(e: &Env, caller: Address, new_admin: Address);

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
        let access_control = default_access_control(&e);

        access_control.initialize(&e, &owner);
        access_control.set_role_admin(&e, YIELD_CONTROLLER_ADMIN_ROLE, DEFAULT_ADMIN_ROLE);
        access_control._grant_role(&e, YIELD_CONTROLLER_ADMIN_ROLE, &admin);

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
        e.storage()
            .instance()
            .set(&ADAPTER_REGISTRY_KEY, &adapter_registry);
        e.storage().instance().set(&CUSD_MANAGER_KEY, &cusd_manager);
        e.storage()
            .instance()
            .set(&YIELD_DISTRIBUTOR_KEY, &yield_distributor);
    }

    fn deposit_collateral(
        e: &Env,
        protocol: Symbol,
        user: Address,
        asset: Address,
        amount: i128,
    ) -> i128 {
        user.require_auth();

        // deps
        let registry_client = adapter_registry_client(&e);
        let cusd_manager_client = cusd_manager_client(&e);
        let adapter =
            LendingAdapterClient::new(e, &registry_client.get_adapter(&YIELD_TYPE.id(), &protocol));
        let asset_client = TokenClient::new(e, &asset);

        // require supported asset for adapter
        let is_asset_supported =
            registry_client.is_supported_asset(&YIELD_TYPE.id(), &protocol, &asset);
        if !is_asset_supported {
            panic!("Asset is not supported by the adapter registry");
        };

        // Deposit collateral
        asset_client.transfer_from(
            &e.current_contract_address(),
            &user,
            &e.current_contract_address(),
            &amount,
        );
        asset_client.approve(
            &e.current_contract_address(),
            &adapter.address,
            &amount,
            &100_u32,
        );
        adapter.deposit(&e.current_contract_address(), &asset, &amount);
        cusd_manager_client.issue_cusd(&e.current_contract_address(), &user, &amount);

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

        // Get the Lending Adapter
        let registry_client = adapter_registry_client(&e);
        let cusd_manager_client = cusd_manager_client(&e);
        let adapter =
            LendingAdapterClient::new(e, &registry_client.get_adapter(&YIELD_TYPE.id(), &protocol));
        let asset_client = TokenClient::new(e, &asset);
        let cusd_token_client = TokenClient::new(e, &cusd_manager_client.get_cusd_id());

        // require supported asset for adapter
        let is_asset_supported =
            registry_client.is_supported_asset(&YIELD_TYPE.id(), &protocol, &asset);
        if !is_asset_supported {
            panic!("Asset is not supported by the adapter registry");
        };

        // Withdraw collateral
        adapter.withdraw(&user, &asset, &amount);

        // Burn cUSD
        cusd_token_client.transfer_from(
            &e.current_contract_address(),
            &user,
            &e.current_contract_address(),
            &amount,
        );
        cusd_token_client.approve(
            &e.current_contract_address(),
            &cusd_manager_client.address,
            &amount,
            &100_u32,
        );
        cusd_manager_client.burn_cusd(
            &e.current_contract_address(),
            &e.current_contract_address(),
            &amount,
        );

        // Transfer asset to user
        asset_client.transfer(&e.current_contract_address(), &user, &amount);

        LendingYieldControllerEvents::withdraw_collateral(&e, user, asset, amount);

        amount
    }

    fn get_yield(e: &Env) -> i128 {
        let registry_client = adapter_registry_client(e);
        let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&YIELD_TYPE.id());
        let user = e.current_contract_address();

        // Use fold to accumulate the total yield across all adapters and assets
        lend_protocols_with_assets.iter().fold(
            0,
            |adapter_acc, (adapter_address, supported_assets)| {
                let adapter_client = LendingAdapterClient::new(e, &adapter_address);

                let adapter_total = supported_assets.iter().fold(0, |asset_acc, asset| {
                    let asset_yield = adapter_client.get_yield(&user, &asset);
                    asset_acc + asset_yield
                });
                adapter_acc + adapter_total
            },
        )
    }

    fn claim_yield(e: &Env) -> i128 {
        let total_yield = Self::get_yield(e);

        if total_yield <= 0 {
            return 0;
        }

        let distributor = distributor_client(e);
        if !distributor.is_distribution_available() {
            panic!("Distribution not ready yet");
        }

        let mut total_claimed: i128 = 0;
        let registry_client = adapter_registry_client(e);
        let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&YIELD_TYPE.id());
        for (adapter_address, supported_assets) in lend_protocols_with_assets.iter() {
            let adapter_client = LendingAdapterClient::new(e, &adapter_address);

            for asset in supported_assets.iter() {
                let token_client = TokenClient::new(e, &asset);
                let claimed = adapter_client.claim_yield(&e.current_contract_address(), &asset);
                log!(&e, "address: {}, claimed: {}", asset, claimed);

                if claimed > 0 {
                    token_client.approve(
                        &e.current_contract_address(),
                        &distributor.address,
                        &claimed,
                        &100_u32,
                    );
                    distributor.distribute_yield(&e.current_contract_address(), &asset, &claimed);
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

    fn set_yield_distributor(e: &Env, yield_distributor: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(e, &e.current_contract_address(), YIELD_CONTROLLER_ADMIN_ROLE);
        set_yield_distributor(e, yield_distributor.clone());
        LendingYieldControllerEvents::set_yield_distributor(e, yield_distributor.clone());
    }

    fn set_adapter_registry(e: &Env, adapter_registry: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(e, &e.current_contract_address(), YIELD_CONTROLLER_ADMIN_ROLE);
        set_adapter_registry(e, adapter_registry.clone());
        LendingYieldControllerEvents::set_adapter_registry(e, adapter_registry.clone());
    }
    
    fn set_cusd_manager(e: &Env, cusd_manager: Address) {
        let access_control = default_access_control(e);
        access_control.only_role(e, &e.current_contract_address(), YIELD_CONTROLLER_ADMIN_ROLE);
        set_cusd_manager(e, cusd_manager.clone());
        LendingYieldControllerEvents::set_cusd_manager(e, cusd_manager.clone());
    }

    fn get_yield_distributor(e: &Env) -> Address {
        get_yield_distributor(e)
    }

    fn get_adapter_registry(e: &Env) -> Address {
        get_adapter_registry(e)
    }

    fn get_cusd_manager(e: &Env) -> Address {
        get_cusd_manager(e)
    }


    fn set_admin(e: &Env, caller: Address, new_admin: Address) {
        let access_control = default_access_control(e);
        access_control.grant_role(e, caller, YIELD_CONTROLLER_ADMIN_ROLE, &new_admin);
        LendingYieldControllerEvents::set_admin(e, new_admin);
    }
}
