use soroban_sdk::{contract, contractimpl, contractmeta, Address, Env, Vec};

use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::admin::{read_administrator, write_administrator};
use crate::asset::{add_asset, remove_asset, verify_if_supported_asset};
use crate::collateral::{receive_collateral_balance, spend_balance};
use crate::manager::{read_cusd_manager, write_cusd_manager};

contractmeta!(key = "Description", val = "Coopstable cUSD manager contract");

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}


#[contract]
pub struct CusdManager;

// manager manages collateral assets supported by this contract
pub trait CusdManagerTrait {
    fn __constructor(e: Env, admin: Address, new_manager: Address, supported_assets: Vec<Address>);
    fn set_admin(e: Env, admin: Address);
    fn set_cusd_manager(e: Env, new_manager: Address);
    fn support_asset(e: Env, asset_address: Address);
    fn remove_asset(e: Env, asset_address: Address);
    fn is_supported_asset(e: Env, asset_address: Address) -> bool;
    fn mint_cusd(e: Env, owner: Address, asset_address: Address, amount: i128);
    fn burn_cusd(e: Env, owner: Address, asset_address: Address, amount: i128);
}

#[contractimpl]
impl CusdManagerTrait for CusdManager {
    fn __constructor(e: Env, admin: Address, manager: Address, supported_assets: Vec<Address>) {
        let admin = admin;
        write_administrator(&e, &admin);
        write_cusd_manager(&e, &manager);
        for asset in supported_assets {
            add_asset(&e, &asset);
        }
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_administrator(&e, &new_admin);
        e.events().publish(("CUSD_MANAGER", "set_admin"), new_admin);
    }

    fn set_cusd_manager(e: Env, new_manager: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();

        e.storage()
            .instance()
            .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

        write_cusd_manager(&e, &new_manager);
        e.events().publish(("CUSD_MANAGER", "set_manager"), new_manager);
    }

    fn support_asset(e: Env, asset_address: Address) {
        let manager = read_cusd_manager(&e);
        manager.require_auth();
        add_asset(&e, &asset_address);
        e.events().publish(("CUSD_MANAGER", "support_asset"), asset_address);
    }

    fn remove_asset(e: Env, asset_address: Address) {
        let manager = read_cusd_manager(&e);
        manager.require_auth();
        remove_asset(&e, &asset_address);
        e.events().publish(("CUSD_MANAGER", "remove_asset"), asset_address);
    }

    fn mint_cusd(e: Env, owner: Address, asset_address: Address, amount: i128) {
        let manager = read_cusd_manager(&e);
        manager.require_auth();
        verify_if_supported_asset(&e, &asset_address);
        receive_collateral_balance(
            &e,
            owner.clone(),
            asset_address.clone(),
            amount
        );
        e.events().publish(("CUSD_MANAGER", "mint_cusd"), owner);
    }

    fn burn_cusd(e: Env, owner: Address, asset_address: Address, amount: i128) {
        let manager = read_cusd_manager(&e);
        manager.require_auth();
        verify_if_supported_asset(&e, &asset_address);
        spend_balance(&e, owner.clone(), asset_address.clone(), amount);
        e.events().publish(("CUSD_MANAGER", "burn_cusd"), owner);
    }

    fn is_supported_asset(e: Env, asset_address: Address) -> bool {
        verify_if_supported_asset(&e, &asset_address)
    }
}