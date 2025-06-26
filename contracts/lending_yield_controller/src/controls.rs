use soroban_sdk::{panic_with_error, vec, Address, Env, IntoVal, Symbol, Vec};
use yield_adapter::lending_adapter::LendingAdapterClient;
use crate::error::LendingYieldControllerError;
use crate::events::LendingYieldControllerEvents;
use crate::utils;
use crate::{storage, storage_types};

pub fn process_deposit(e: &Env, protocol: &Symbol, user: Address, asset: Address, amount: i128) -> i128 { 
    let registry_client = storage::adapter_registry_client(&e);
    let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
    if let Some((id, fn_name, args)) = adapter.deposit_auth(&user, &asset, &amount) {
        utils::authenticate_contract(
            &e, 
            id, 
            fn_name,  
            args,
        );
    }
    utils::authenticate_contract(
        &e, 
        adapter.address.clone(), 
        Symbol::new(&e, "deposit"),
        vec![
            e,
            (&user).into_val(e),
            (&asset).into_val(e),
            (&amount).into_val(e),
        ]
    );
    
    let deposited = adapter.deposit(
        &user, 
        &asset, 
        &amount
    );

    process_cusd_issue(e, user.clone(), amount);

    deposited
}

fn process_cusd_issue(e: &Env, user: Address, amount: i128) {
    let cusd_manager_client = storage::cusd_manager_client(&e);
    utils::authenticate_contract(
        &e, 
        cusd_manager_client.address.clone(), 
        Symbol::new(&e, "issue_cusd"), 
        vec![
            e,
            (&cusd_manager_client.address).into_val(e),
            (&amount).into_val(e),
        ]
    );
    cusd_manager_client.issue_cusd(&user, &amount);
}

fn process_cusd_burn(e: &Env, user: Address, amount: i128) {
    let cusd_manager_client = storage::cusd_manager_client(&e);
    cusd_manager_client.burn_cusd(&user, &amount);
}

pub fn process_withdraw_collateral(e: &Env, protocol: &Symbol, user: Address, asset: Address, amount: i128) -> i128 {
    
    let registry_client = storage::adapter_registry_client(&e);
    
    let adapter =
        LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
    
    let is_asset_supported =
        registry_client.is_supported_asset(&storage_types::YIELD_TYPE.id(), &protocol, &asset);
    if !is_asset_supported {
        panic_with_error!(e, LendingYieldControllerError::UnsupportedAsset);
    };
    
    if let Some((id, fn_name, args)) = adapter.withdraw_auth(&user, &asset, &amount) {
        utils::authenticate_contract(
            &e, 
            id, 
            fn_name,  
            args,
        );
    }
    utils::authenticate_contract(
        &e, 
        adapter.address.clone(), 
        Symbol::new(&e, "withdraw"), 
        vec![
            e,
            (&user).into_val(e),
            (&asset).into_val(e),
            (&amount).into_val(e),
        ]
    );
    
    let withdrawn = adapter.withdraw(&user, &asset, &amount);
    
    process_cusd_burn(e, user.clone(), amount);

    withdrawn
}

pub fn process_distribute_cusd_yield(e: &Env, asset: Address, amount: i128) {
    let distributor = storage::distributor_client(e);
    utils::authenticate_contract(
        &e, 
        distributor.address.clone(), 
        Symbol::new(&e, "distribute_yield"), 
        vec![
            e,
            (&asset).into_val(e),
            (&amount).into_val(e),
        ]
    );
    distributor.distribute_yield(&asset, &amount); 
}

pub fn read_yield(e: &Env) -> i128 {
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

pub fn process_claim_and_distribute_yield(e: &Env) -> i128 {
    let mut claimed_total: i128 = 0;
    let registry_client = storage::adapter_registry_client(e);
    let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&storage_types::YIELD_TYPE.id());
    
    for (adapter_address, supported_assets) in lend_protocols_with_assets.iter() {
        claimed_total += process_claim_yield(e, &adapter_address, supported_assets.clone());
    }
    claimed_total
}

pub fn process_claim_emissions(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    let registry_client = storage::adapter_registry_client(e);
    let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
    let emissions = adapter.get_emissions(&asset);
    if emissions > 0 {
        let distributor = storage::distributor_client(e);
        if let Some((id, fn_name, args)) = adapter.claim_emissions_auth(&distributor.address, &asset) {
            utils::authenticate_contract(
                &e, 
                id, 
                fn_name,  
                args,
            );
        }
        utils::authenticate_contract(
            &e, 
            adapter.address.clone(), 
            Symbol::new(&e, "claim_emissions"),
            vec![
                e,
                (&distributor.address).into_val(e),
                (&asset).into_val(e),
            ]
        );
        let claimed = adapter.claim_emissions(&distributor.address, &asset);
        LendingYieldControllerEvents::claim_emissions(
            &e,
            distributor.address.clone(),
            asset.clone(),
            claimed,
        );            
    }
    0
}

pub fn read_emissions(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    let registry_client = storage::adapter_registry_client(e);
    let adapter = LendingAdapterClient::new(e, &registry_client.get_adapter(&storage_types::YIELD_TYPE.id(), &protocol));
    adapter.get_emissions(&asset)
}

fn process_claim_yield(e: &Env, adapter_address: &Address, supported_assets: Vec<Address>) -> i128 {
    let distributor = storage::distributor_client(e);
    let cusd_manager = storage::cusd_manager_client(e);
    let adapter = LendingAdapterClient::new(e, adapter_address);
    let mut adapter_claimed_total: i128 = 0;
    
    for asset in supported_assets.iter() {
        
        let yield_amount = adapter.get_yield(&asset);
        
        if yield_amount > 0 {

            authenticate_for_claim_yield(e, adapter_address, asset.clone(), yield_amount);

            let claimed = adapter.claim_yield(&asset, &yield_amount);

            let deposited = process_deposit_for_claim(e, adapter_address.clone(),asset.clone(), claimed);
            
            process_cusd_issue(e, distributor.address.clone(), deposited); 
            
            process_distribute_cusd_yield(e, cusd_manager.get_cusd_id(), deposited);
            
            adapter_claimed_total += deposited;

            LendingYieldControllerEvents::claim_yield(
                &e,
                e.current_contract_address(),
                asset.clone(),
                claimed,
            );
        }
    }
    
    adapter_claimed_total
}

fn authenticate_for_claim_yield(e: &Env, adapter_address: &Address, asset: Address, yield_amount: i128) {
    let adapter = LendingAdapterClient::new(e, &adapter_address);
    if let Some((pool_id, fn_name, args)) = adapter.claim_yield_auth(&asset, &yield_amount) {
        utils::authenticate_contract(
            &e, 
            pool_id.clone(), 
            fn_name,  
            args,
        );
    }
    utils::authenticate_contract(
        &e, 
        adapter.address.clone(), 
        Symbol::new(&e, "claim_yield"),
        vec![
            e,
            (&asset).into_val(e),
            (&yield_amount).into_val(e),
        ]
    );
}

fn process_deposit_for_claim(e: &Env, protocol_id: Address, asset: Address, yield_amount: i128) -> i128 { 
    
    let adapter = LendingAdapterClient::new(e, &protocol_id);

    if let Some((pool_id, fn_name, args)) = adapter.deposit_auth(&e.current_contract_address(), &asset, &yield_amount) {
        utils::authenticate_contract(
            &e, 
            pool_id.clone(), 
            fn_name,  
            args,
        );

        utils::authenticate_contract( // authenticate yield controller for depositing withdrewn tokens in the pool
            &e, 
            asset.clone(), 
            Symbol::new(&e, "transfer"), 
            vec![
                e,
                (&e.current_contract_address()).into_val(e),
                (&pool_id).into_val(e),
                (&yield_amount).into_val(e),
            ]
        );
    }

    utils::authenticate_contract(
        &e, 
        adapter.address.clone(), 
        Symbol::new(&e, "deposit"),
        vec![
            e,
            (&e.current_contract_address()).into_val(e),
            (&asset).into_val(e),
            (&yield_amount).into_val(e),
        ]
    );

    let deposited = adapter.deposit(
        &e.current_contract_address(), 
        &asset, 
        &yield_amount
    );

    deposited
}
