use soroban_sdk::{panic_with_error, token::TokenClient, vec, Address, Env, IntoVal, Symbol, Val, Vec};
use yield_adapter::lending_adapter::LendingAdapterClient;
use crate::error::LendingYieldControllerError;
use crate::events::LendingYieldControllerEvents;
use crate::storage_types::{HarvestState, PendingHarvest};
use crate::utils;
use crate::{storage, storage_types};

/// A yield below this many stroops is treated as no yield. Blend rejects a supply that mints zero bTokens,
/// so a harvest of less than `b_rate / 1e12` stroops could never be recompounded. 1_000 holds while
/// `b_rate` is under 1000.0; raise it before a reserve gets anywhere near that.
pub(crate) const MIN_HARVEST: i128 = 1_000;

fn adapter_for<'a>(e: &'a Env, protocol: &Symbol) -> LendingAdapterClient<'a> {
    LendingAdapterClient::new(e, &storage::adapter_registry_client(e).get_adapter(&storage_types::YIELD_TYPE.id(), protocol))
}

// Collateral moves mint/burn cUSD 1:1, so only registry-supported assets may reach the adapter.
fn supported_asset_adapter<'a>(e: &'a Env, protocol: &Symbol, asset: &Address) -> LendingAdapterClient<'a> {
    if !storage::adapter_registry_client(e).is_supported_asset(&storage_types::YIELD_TYPE.id(), protocol, asset) {
        panic_with_error!(e, LendingYieldControllerError::UnsupportedAsset);
    }
    adapter_for(e, protocol)
}

fn require_positive(e: &Env, amount: i128) {
    if amount <= 0 {
        panic_with_error!(e, LendingYieldControllerError::NegativeAmountError);
    }
}

// The controller calls the adapter, the manager and the distributor directly, and a direct caller needs no
// auth entry. Only calls made one level further down are pre-authorized: the pool's `submit` and `claim`,
// and the token transfer the pool makes when the controller itself is the spender.
fn authorize(e: &Env, call: Option<(Address, Symbol, Vec<Val>)>) {
    if let Some((contract, fn_name, args)) = call {
        utils::authenticate_contract(e, contract, fn_name, args);
    }
}

/// Supplies `amount` of `asset` from `from` into the pool position. Mints nothing.
/// `from` must end up exactly `amount` poorer, whatever the pool and adapter report.
fn supply(e: &Env, adapter: &LendingAdapterClient, from: &Address, asset: &Address, amount: i128) {
    let token = TokenClient::new(e, asset);
    let before = token.balance(from);

    if let Some((pool, fn_name, args)) = adapter.deposit_auth(from, asset, &amount) {
        if *from == e.current_contract_address() {
            utils::authenticate_contract(
                e,
                asset.clone(),
                Symbol::new(e, "transfer"),
                vec![e, from.into_val(e), (&pool).into_val(e), (&amount).into_val(e)],
            );
        }
        utils::authenticate_contract(e, pool, fn_name, args);
    }
    // The adapter returns the amount it was asked for, not what moved; the balance check is what counts.
    let _ = adapter.deposit(from, asset, &amount);

    if before - token.balance(from) != amount {
        panic_with_error!(e, LendingYieldControllerError::BalanceError);
    }
}

/// Withdraws `amount` of `asset` from the pool position to `to`. Burns nothing.
/// `to` must receive exactly `amount`: Blend clamps a withdrawal larger than the position and pays less.
fn withdraw(e: &Env, adapter: &LendingAdapterClient, to: &Address, asset: &Address, amount: i128) {
    let token = TokenClient::new(e, asset);
    let before = token.balance(to);

    authorize(e, adapter.withdraw_auth(to, asset, &amount));
    let _ = adapter.withdraw(to, asset, &amount);

    if token.balance(to) - before != amount {
        panic_with_error!(e, LendingYieldControllerError::BalanceError);
    }
}

pub fn process_deposit(e: &Env, protocol: &Symbol, user: Address, asset: Address, amount: i128) -> i128 {
    if storage::read_paused(e).0 {
        panic_with_error!(e, LendingYieldControllerError::MintPaused);
    }
    require_positive(e, amount);
    let adapter = supported_asset_adapter(e, protocol, &asset);
    supply(e, &adapter, &user, &asset, amount);
    storage::cusd_manager_client(e).issue_cusd(&user, &amount);
    amount
}

pub fn process_withdraw_collateral(e: &Env, protocol: &Symbol, user: Address, asset: Address, amount: i128) -> i128 {
    if storage::read_paused(e).1 {
        panic_with_error!(e, LendingYieldControllerError::RedeemPaused);
    }
    require_positive(e, amount);
    let adapter = supported_asset_adapter(e, protocol, &asset);
    withdraw(e, &adapter, &user, &asset, amount);
    storage::cusd_manager_client(e).burn_cusd(&user, &amount);
    amount
}

/// Tops up the backing of a supported asset from `from`. No cUSD is minted, so it works while mint is paused.
pub fn process_recollateralize(e: &Env, protocol: &Symbol, asset: Address, from: Address, amount: i128) {
    require_positive(e, amount);
    let adapter = supported_asset_adapter(e, protocol, &asset);
    supply(e, &adapter, &from, &asset, amount);
    LendingYieldControllerEvents::recollateralize(e, protocol.clone(), asset, from, amount);
}

/// Withdraws a reserve that is not a supported asset out of the pool position. It can never touch backing.
pub fn process_rescue_collateral(e: &Env, protocol: &Symbol, asset: Address, to: Address, amount: i128) {
    require_positive(e, amount);
    let adapter = adapter_for(e, protocol);
    // Two barriers: the registry allowlist, and the asset the adapter itself is bound to. The second one
    // holds even while the allowlist is empty, as it is right after an adapter is registered.
    if storage::adapter_registry_client(e).is_supported_asset(&storage_types::YIELD_TYPE.id(), protocol, &asset)
        || adapter.get_asset() == asset
    {
        panic_with_error!(e, LendingYieldControllerError::AssetIsSupported);
    }
    withdraw(e, &adapter, &to, &asset, amount);
    LendingYieldControllerEvents::rescue_collateral(e, protocol.clone(), asset, to, amount);
}

/// Transfers tokens the controller holds itself. Refused while a harvest of that asset sits between
/// stage 1 and stage 2, because those tokens are yield on their way back to the pool.
pub fn process_rescue(e: &Env, asset: Address, to: Address, amount: i128) {
    require_positive(e, amount);
    if storage::read_held_harvest(e, &asset) > 0 {
        panic_with_error!(e, LendingYieldControllerError::HarvestAlreadyInProgress);
    }
    TokenClient::new(e, &asset).transfer(&e.current_contract_address(), &to, &amount);
    LendingYieldControllerEvents::rescue(e, asset, to, amount);
}

pub fn read_yield(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    adapter_for(e, protocol).get_yield(&asset)
}

pub fn read_apy(e: &Env, protocol: &Symbol, asset: Address) -> u32 {
    adapter_for(e, protocol).get_apy(&asset)
}

pub fn read_emissions(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    adapter_for(e, protocol).get_emissions(&asset)
}

/// Claims the pool's emissions for `asset` to the distributor's treasury. Returns the amount claimed.
pub fn process_claim_emissions(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    let adapter = supported_asset_adapter(e, protocol, &asset);
    if adapter.get_emissions(&asset) <= 0 {
        return 0;
    }
    let treasury = storage::distributor_client(e).get_treasury();
    authorize(e, adapter.claim_emissions_auth(&treasury, &asset));
    let claimed = adapter.claim_emissions(&treasury, &asset);
    LendingYieldControllerEvents::claim_emissions(e, treasury, asset, claimed);
    claimed
}

// =========================================================================
// Multi-stage yield claiming functions
//
// The adapter's principal (`get_principal`) is what is owed to cUSD holders. It moves only with deposits,
// withdrawals and minted yield, and is never overwritten from the live value of the position. So a loss
// is not forgotten at an epoch reset: yield reads 0 until the position is worth more than what is owed.
// =========================================================================

/// Stage 1: Harvest yield from the lending protocol
/// Withdraws accumulated yield and stores it for later processing.
pub fn process_harvest_yield(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    if storage::has_pending_harvest(e, protocol, &asset) {
        panic_with_error!(e, LendingYieldControllerError::HarvestAlreadyInProgress);
    }
    let adapter = supported_asset_adapter(e, protocol, &asset);

    let yield_amount = adapter.get_yield(&asset);
    if yield_amount < MIN_HARVEST {
        // No yield worth harvesting: advance the epoch if a distribution is due, so the system doesn't stall.
        let distributor = storage::distributor_client(e);
        if distributor.is_distribution_available() {
            distributor.advance_epoch();
            // Fold the epoch's deposits and withdrawals into the principal. The amount owed is unchanged.
            adapter.update_epoch_principal(&asset, &distributor.get_current_epoch(), &adapter.get_principal(&asset));
            LendingYieldControllerEvents::harvest_yield(e, protocol.clone(), asset, 0);
        }
        return 0;
    }

    // As with a withdrawal, the controller must receive exactly what it asked for.
    let token = TokenClient::new(e, &asset);
    let before = token.balance(&e.current_contract_address());
    authorize(e, adapter.claim_yield_auth(&asset, &yield_amount));
    let _ = adapter.claim_yield(&asset, &yield_amount);
    if token.balance(&e.current_contract_address()) - before != yield_amount {
        panic_with_error!(e, LendingYieldControllerError::BalanceError);
    }
    let claimed = yield_amount;

    storage::set_harvest_adapter(e, protocol, &asset, &adapter.address);
    storage::set_pending_harvest(e, &PendingHarvest {
        protocol: protocol.clone(),
        asset: asset.clone(),
        amount: claimed,
        state: HarvestState::Harvested,
    });
    storage::add_held_harvest(e, &asset, claimed);

    LendingYieldControllerEvents::harvest_yield(e, protocol.clone(), asset, claimed);

    claimed
}

fn pending_harvest(e: &Env, protocol: &Symbol, asset: &Address) -> PendingHarvest {
    storage::get_pending_harvest(e, protocol, asset)
        .unwrap_or_else(|| panic_with_error!(e, LendingYieldControllerError::NoPendingHarvest))
}

// The adapter that took the pending harvest out of the pool. A harvest that was pending before this was
// recorded falls back to the registered adapter.
fn harvest_adapter<'a>(e: &'a Env, protocol: &Symbol, asset: &Address) -> LendingAdapterClient<'a> {
    match storage::get_harvest_adapter(e, protocol, asset) {
        Some(address) => LendingAdapterClient::new(e, &address),
        None => adapter_for(e, protocol),
    }
}

// Stages 2 and 3 only continue on the adapter that did stage 1. If the registry now names another one,
// the harvest has to be cancelled, which puts it back through the original adapter.
fn continuing_adapter<'a>(e: &'a Env, protocol: &Symbol, asset: &Address) -> LendingAdapterClient<'a> {
    let adapter = supported_asset_adapter(e, protocol, asset);
    if storage::has_pending_harvest(e, protocol, asset) && harvest_adapter(e, protocol, asset).address != adapter.address {
        panic_with_error!(e, LendingYieldControllerError::InvalidHarvestState);
    }
    adapter
}

// Puts a stage-1 harvest back into the pool: stage 2, and a cancel before stage 2.
fn redeposit_harvest(e: &Env, adapter: &LendingAdapterClient, pending: &PendingHarvest) {
    supply(e, adapter, &e.current_contract_address(), &pending.asset, pending.amount);
    storage::add_held_harvest(e, &pending.asset, -pending.amount);
}

/// Stage 2: Recompound harvested yield back into the protocol
/// Re-deposits the harvested yield to continue earning interest.
pub fn process_recompound_yield(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    let adapter = continuing_adapter(e, protocol, &asset);
    let pending = pending_harvest(e, protocol, &asset);
    if pending.state != HarvestState::Harvested {
        panic_with_error!(e, LendingYieldControllerError::InvalidHarvestState);
    }

    redeposit_harvest(e, &adapter, &pending);
    storage::set_pending_harvest(e, &PendingHarvest { state: HarvestState::Recompounded, ..pending.clone() });

    LendingYieldControllerEvents::recompound_yield(e, protocol.clone(), asset, pending.amount);

    pending.amount
}

/// Stage 3: Finalize distribution of yield to members
/// Issues cUSD and distributes to treasury and members.
pub fn process_finalize_distribution(e: &Env, protocol: &Symbol, asset: Address) -> i128 {
    let adapter = continuing_adapter(e, protocol, &asset);
    let pending = pending_harvest(e, protocol, &asset);
    if pending.state != HarvestState::Recompounded {
        panic_with_error!(e, LendingYieldControllerError::InvalidHarvestState);
    }

    // Mint only what is still surplus. The pool's rounding in stages 1 and 2 costs a stroop or two, and a
    // loss since stage 1 can cost all of it; cUSD is never issued against yield that is no longer there.
    // With nothing left, the admin drops the harvest with `cancel_harvest`.
    let yield_amount = pending.amount.min(adapter.get_yield(&asset));
    if yield_amount <= 0 {
        panic_with_error!(e, LendingYieldControllerError::YieldUnavailable);
    }

    let distributor = storage::distributor_client(e);
    let cusd_manager = storage::cusd_manager_client(e);

    cusd_manager.issue_cusd(&distributor.address, &yield_amount);
    // The distributor returns 0 and moves nothing when no distribution is due.
    if distributor.distribute_yield(&cusd_manager.get_cusd_id(), &yield_amount) != yield_amount {
        panic_with_error!(e, LendingYieldControllerError::YieldUnavailable);
    }

    // The minted yield is now owed too. Anything the position earned since stage 1 stays yield.
    // `distribute_yield` has already moved the distributor to the next epoch.
    adapter.update_epoch_principal(
        &asset,
        &distributor.get_current_epoch(),
        &(adapter.get_principal(&asset) + yield_amount),
    );

    storage::remove_pending_harvest(e, protocol, &asset);

    LendingYieldControllerEvents::finalize_distribution(e, protocol.clone(), asset, yield_amount);

    yield_amount
}

/// Cancel a pending harvest operation. Before stage 2 the harvested yield goes back into the pool first,
/// so a cancel never leaves tokens in the controller or lets the same yield be withdrawn twice.
pub fn process_cancel_harvest(e: &Env, protocol: &Symbol, asset: Address) {
    let pending = pending_harvest(e, protocol, &asset);
    if pending.state == HarvestState::Harvested {
        // No registry lookup: a cancel must work even after the asset or the adapter was removed or swapped.
        redeposit_harvest(e, &harvest_adapter(e, protocol, &asset), &pending);
    }
    storage::remove_pending_harvest(e, protocol, &asset);

    LendingYieldControllerEvents::cancel_harvest(e, protocol.clone(), asset);
}
