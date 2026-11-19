#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::{
    contract::{BlendCapitalAdapter, BlendCapitalAdapterClient},
    mocks::blend_pool_mock::{PoolContract, PoolContractClient},
    constants::USER_DEPOSITS,
};
use yield_adapter::lending_adapter::LendingAdapterClient;

struct TestFixture {
    env: Env,
    adapter: BlendCapitalAdapterClient<'static>,
    pool: PoolContractClient<'static>,
    yield_controller: Address,
    user1: Address,
    user2: Address,
    usdc_token_id: Address,
    blend_token_id: Address,
    token_admin: Address,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        env.ledger().set_sequence_number(100);

        let token_admin = Address::generate(&env);
        let yield_controller = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let usdc_token_id = usdc_token.address();
        let blend_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let blend_token_id = blend_token.address();

        let pool_id = env.register(PoolContract, ());
        let pool = PoolContractClient::new(&env, &pool_id);

        pool.init(&usdc_token_id);
        let adapter_id = env.register(
            BlendCapitalAdapter,
            (yield_controller.clone(), pool_id.clone(), blend_token_id.clone(), Address::generate(&env), usdc_token_id.clone(), 0i128),
        );
        let adapter = BlendCapitalAdapterClient::new(&env, &adapter_id);

        TestFixture {
            env,
            adapter,
            pool,
            yield_controller,
            user1,
            user2,
            usdc_token_id,
            blend_token_id,
            token_admin,
        }
    }

    fn lending_adapter_client(&self) -> LendingAdapterClient<'static> {
        LendingAdapterClient::new(&self.env, &self.adapter.address)
    }

    fn assert_event_with_address_tuple_data(&self, expected_event: (Symbol, Address), expected_data: (Address, i128)) {
        let events = self.env.events().all();
        if !events.events().is_empty() {
            let published_event = events.filter_by_contract(&self.adapter.address);
            let expected = vec![
                &self.env,
                (
                    self.adapter.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn assert_event_with_user_tuple_data(&self, expected_event: (Symbol, Address, Address), expected_data: (Address, i128)) {
        let events = self.env.events().all();
        if !events.events().is_empty() {
            let published_event = events.filter_by_contract(&self.adapter.address);
            let expected = vec![
                &self.env,
                (
                    self.adapter.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn get_contract_deposit(&self, asset: &Address) -> Option<i128> {
        self.env.as_contract(&self.adapter.address, || {
            let key = (USER_DEPOSITS, self.yield_controller.clone(), asset.clone());
            let amount: i128 = self.env.storage().instance().get(&key).unwrap_or(0);
            if amount > 0 { Some(amount) } else { None }
        })
    }

    fn update_pool_b_rate(&self, asset: &Address, new_b_rate: i128) {
        self.pool.update_b_rate(asset, &new_b_rate);
    }
}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();

    fixture.env.as_contract(&fixture.adapter.address, || {
        let stored_controller: Address = fixture.env
            .storage()
            .instance()
            .get(&Symbol::new(&fixture.env, "LACID"))
            .unwrap();
        let stored_pool: Address = fixture.env
            .storage()
            .instance()
            .get(&Symbol::new(&fixture.env, "LID"))
            .unwrap();

        assert_eq!(stored_controller, fixture.yield_controller);
        assert_eq!(stored_pool, fixture.pool.address);
    });
}

#[test]
fn test_deposit() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    let _ = fixture.env.events().all();

    let result = client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    assert_eq!(result, amount);
    fixture.assert_event_with_address_tuple_data(
        (Symbol::new(&fixture.env, "deposit"), fixture.adapter.address.clone()),
        (fixture.usdc_token_id.clone(), amount),
    );

    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, Some(amount));
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_deposit_unauthorized() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_auths(&[]);

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);
}

#[test]
fn test_withdraw() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let deposit_amount = 1000_0000000;
    let withdraw_amount = 500_0000000;

    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    let _ = fixture.env.events().all();

    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw_amount);

    assert_eq!(result, withdraw_amount);
    fixture.assert_event_with_user_tuple_data(
        (Symbol::new(&fixture.env, "withdraw"), fixture.adapter.address.clone(), fixture.user1.clone()),
        (fixture.usdc_token_id.clone(), withdraw_amount),
    );

    let remaining_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(remaining_deposit, Some(deposit_amount - withdraw_amount));
}

#[test]
fn test_withdraw_full_amount() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &amount);

    assert_eq!(result, amount);
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, None);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_withdraw_unauthorized() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_auths(&[]);

    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &amount);
}

#[test]
fn test_get_yield_no_accrual() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0);
}

#[test]
fn test_claim_yield_no_yield() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    let claimed_yield = client.claim_yield(&fixture.usdc_token_id, &yield_amount);
    assert_eq!(claimed_yield, 0);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_claim_yield_unauthorized() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_auths(&[]);
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    client.claim_yield(&fixture.usdc_token_id, &yield_amount);
}

#[test]
fn test_authorization_requirements() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);
    let auths = fixture.env.auths();
    assert!(!auths.is_empty(), "No authorizations were recorded");

    let yield_controller_auth = auths.iter().find(|(addr, _)| *addr == fixture.yield_controller);
    assert!(
        yield_controller_auth.is_some(),
        "Yield controller authorization was not required"
    );
}

#[test]
fn test_multiple_deposits() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let deposit1 = 500_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit1);

    let deposit2 = 300_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit2);
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, Some(deposit1 + deposit2));
}

#[test]
fn test_multiple_operations() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let deposit1 = 500_0000000;
    let deposit2 = 300_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit1);
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit2);

    let withdraw1 = 200_0000000;
    let withdraw2 = 300_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw1);
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw2);
    let expected_remaining = deposit1 + deposit2 - withdraw1 - withdraw2;
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, Some(expected_remaining));

    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &expected_remaining);
    let final_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(final_deposit, None);
}

#[test]
fn test_multiple_users() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let deposit1 = 500_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit1);

    let deposit2 = 300_0000000;
    client.deposit(&fixture.user2, &fixture.usdc_token_id, &deposit2);
    let total_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(total_deposit, Some(deposit1 + deposit2));

    let withdraw1 = 200_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw1);

    client.withdraw(&fixture.user2, &fixture.usdc_token_id, &deposit2);
    let remaining_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(remaining_deposit, Some(deposit1 - withdraw1));
}

#[test]
fn test_negative_yield_handling() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);
    let new_b_rate = 900_000_000_000; // 10% loss
    fixture.update_pool_b_rate(&fixture.usdc_token_id, new_b_rate);

    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0, "Negative yield should be reported as 0");

    let claimed_yield = client.claim_yield(&fixture.usdc_token_id, &yield_amount);
    assert_eq!(claimed_yield, 0, "Claiming negative yield should return 0");
}

#[test]
fn test_zero_amount_operations() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let result = client.deposit(&fixture.user1, &fixture.usdc_token_id, &0);
    assert_eq!(result, 0);

    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, None);
    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &0);
    assert_eq!(result, 0);
}

// I-03: the adapter is bound to one asset, for good.
#[test]
fn test_deposit_rejects_any_other_asset() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let other_token_id = fixture.env.register_stellar_asset_contract_v2(fixture.token_admin.clone()).address();
    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &1000_0000000);
    assert_eq!(
        client.try_deposit(&fixture.user1, &other_token_id, &500_0000000).err().unwrap(),
        Ok(soroban_sdk::Error::from_contract_error(1300))
    );
    assert_eq!(fixture.get_contract_deposit(&other_token_id), None);

    // The binding is set once: not even the owner can point the adapter at another asset.
    assert_eq!(
        fixture.adapter.try_set_asset(&other_token_id).err().unwrap(),
        Ok(soroban_sdk::Error::from_contract_error(1301))
    );
    assert_eq!(client.get_asset(), fixture.usdc_token_id);
}

// The state of an adapter upgraded from a version without the binding: no asset stored. Deposits fail
// closed until the owner binds it; withdrawals stay open.
#[test]
fn test_deposit_fails_closed_until_an_asset_is_bound() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    fixture.env.mock_all_auths();
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &1000_0000000);
    fixture.env.as_contract(&fixture.adapter.address, || {
        fixture.env.storage().instance().remove(&crate::constants::ASSET)
    });

    assert_eq!(
        client.try_deposit(&fixture.user1, &fixture.usdc_token_id, &1).err().unwrap(),
        Ok(soroban_sdk::Error::from_contract_error(1300))
    );
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &400_0000000);

    fixture.adapter.set_asset(&fixture.usdc_token_id);
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &1);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_asset_unauthorized() {
    let fixture = TestFixture::create();
    fixture.adapter.set_asset(&fixture.usdc_token_id);
}

// The amount owed follows deposits and withdrawals, never the live value of the position.
#[test]
fn test_get_principal_tracks_what_is_owed() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    fixture.env.mock_all_auths();

    assert_eq!(client.get_principal(&fixture.usdc_token_id), 0);
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &1000_0000000);
    fixture.pool.add_yield(&fixture.usdc_token_id, &50_0000000);
    assert_eq!(client.get_principal(&fixture.usdc_token_id), 1000_0000000);
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &400_0000000);
    assert_eq!(client.get_principal(&fixture.usdc_token_id), 600_0000000);
    client.update_epoch_principal(&fixture.usdc_token_id, &1, &600_0000000);
    assert_eq!(client.get_principal(&fixture.usdc_token_id), 600_0000000);
}

// I-05: with nothing owed (or the record over-withdrawn), the whole position is yield.
#[test]
fn test_yield_is_readable_when_nothing_is_owed() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    fixture.env.mock_all_auths();

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &1000_0000000);
    fixture.pool.add_yield(&fixture.usdc_token_id, &50_0000000);
    client.update_epoch_principal(&fixture.usdc_token_id, &1, &-5_0000000);
    assert_eq!(
        client.get_yield(&fixture.usdc_token_id),
        client.get_balance(&fixture.yield_controller, &fixture.usdc_token_id)
    );
}

#[test]
fn test_edge_case_operations() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0);

    let claimed_yield = client.claim_yield(&fixture.usdc_token_id, &yield_amount);
    assert_eq!(claimed_yield, 0);
    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &0);
    assert_eq!(result, 0);
}

#[test]
fn test_epoch_based_yield_tracking() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let deposit_amount = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    let yield_amount = 50_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &yield_amount);
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, yield_amount);

    let epoch_1 = 1u64;
    let principal_after_dist = deposit_amount + yield_amount; // 1050
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_1, &principal_after_dist);
    let yield_after_epoch = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_epoch, 0);

    let additional_yield = 25_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &additional_yield);
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert!((current_yield - additional_yield).abs() <= 1, 
        "Expected yield: {}, got: {}", additional_yield, current_yield);

    let withdrawal_amount = 100_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdrawal_amount);

    // Current balance: 1050 + 25 - 100 = 975
    // Adjusted principal: 1050 - 100 = 950
    // Expected yield: 975 - 950 = 25
    let yield_after_withdrawal = client.get_yield(&fixture.usdc_token_id);
    assert!((yield_after_withdrawal - additional_yield).abs() <= 1, 
        "Expected yield: {}, got: {}", additional_yield, yield_after_withdrawal);

    let large_withdrawal = 200_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &large_withdrawal);

    // Current balance: 975 - 200 = 775
    // Adjusted principal: 1050 - 300 = 750
    // Expected yield: 775 - 750 = 25
    let yield_after_large_withdrawal = client.get_yield(&fixture.usdc_token_id);
    assert!((yield_after_large_withdrawal - additional_yield).abs() <= 2, 
        "Expected yield: {}, got: {}", additional_yield, yield_after_large_withdrawal);
}

#[test]
fn test_epoch_transition_compound_effect() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let initial_deposit = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &initial_deposit);

    let epoch_0_yield = 50_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &epoch_0_yield);
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, epoch_0_yield);

    let epoch_1 = 1u64;
    let epoch_1_principal = initial_deposit + epoch_0_yield; // 1050
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_1, &epoch_1_principal);
    let yield_after_epoch_1 = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_epoch_1, 0);

    let epoch_1_yield = 52_5000000; // 5% of 1050
    fixture.pool.add_yield(&fixture.usdc_token_id, &epoch_1_yield);
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, epoch_1_yield);

    let epoch_2 = 2u64;
    let epoch_2_principal = epoch_1_principal + epoch_1_yield; // 1102.5
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_2, &epoch_2_principal);
    let yield_after_epoch_2 = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_epoch_2, 0);

    assert_eq!(epoch_2_principal, initial_deposit + epoch_0_yield + epoch_1_yield);
}

// ============================================================================
// Zero-yield epoch advancement tests
// ============================================================================

#[test]
fn test_zero_yield_epoch_advancement() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Deposit initial funds
    let deposit_amount = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    // No yield accrued — get_yield returns 0
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0);

    // Advance epoch with same principal (0 yield scenario)
    let epoch_1 = 1u64;
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_1, &deposit_amount);

    // Yield should still be 0 after epoch advance
    let yield_after = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after, 0);

    // Balance unchanged
    let balance = client.get_balance(&fixture.yield_controller, &fixture.usdc_token_id);
    assert_eq!(balance, deposit_amount);

    // Now yield accrues in epoch 1
    let epoch_1_yield = 50_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &epoch_1_yield);
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, epoch_1_yield);
}

#[test]
fn test_multiple_zero_yield_epochs_then_yield() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    let deposit_amount = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    // Advance through 3 epochs with 0 yield each time
    for epoch in 1..=3u64 {
        assert_eq!(client.get_yield(&fixture.usdc_token_id), 0);
        client.update_epoch_principal(&fixture.usdc_token_id, &epoch, &deposit_amount);
    }

    // Principal should still be the original deposit
    let balance = client.get_balance(&fixture.yield_controller, &fixture.usdc_token_id);
    assert_eq!(balance, deposit_amount);

    // Yield finally accrues in epoch 4
    let yield_amount = 100_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &yield_amount);
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, yield_amount);

    // Normal epoch transition works after zero-yield epochs
    let epoch_4 = 4u64;
    let new_principal = deposit_amount + yield_amount;
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_4, &new_principal);
    assert_eq!(client.get_yield(&fixture.usdc_token_id), 0);
}

#[test]
fn test_zero_yield_with_deposits_and_withdrawals() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Initial deposit
    let deposit_amount = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    // Additional deposit during epoch 0 (no yield accrued)
    let additional = 500_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &additional);

    // Yield should be 0 despite balance being higher, because deposits_in_epoch tracks it
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0);

    // Advance to epoch 1 with current balance as new principal
    let total_deposited = deposit_amount + additional;
    let epoch_1 = 1u64;
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_1, &total_deposited);

    // Still 0 yield
    assert_eq!(client.get_yield(&fixture.usdc_token_id), 0);

    // Withdrawal during epoch 1 (still no yield)
    let withdraw = 200_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw);

    // Yield should still be 0 (balance went down due to withdrawal, not yield)
    assert_eq!(client.get_yield(&fixture.usdc_token_id), 0);

    // Advance to epoch 2 with current balance
    let current_balance = total_deposited - withdraw;
    let epoch_2 = 2u64;
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_2, &current_balance);
    assert_eq!(client.get_yield(&fixture.usdc_token_id), 0);
}
#[test]
fn test_member_paths_leave_ttl_alone_and_admin_paths_extend_it() {
    use soroban_sdk::testutils::storage::Instance;
    use yield_adapter::constants::ADAPTER_INSTANCE_BUMP_AMOUNT;

    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    fixture.env.mock_all_auths();
    let ttl = || fixture.env.as_contract(&fixture.adapter.address, || fixture.env.storage().instance().get_ttl());

    let initial = ttl();
    assert!(initial < ADAPTER_INSTANCE_BUMP_AMOUNT);
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &1000_0000000);
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &400_0000000);
    assert_eq!(ttl(), initial, "a member's deposit or withdrawal must not pay rent");

    client.update_epoch_principal(&fixture.usdc_token_id, &1, &600_0000000);
    assert_eq!(ttl(), ADAPTER_INSTANCE_BUMP_AMOUNT);
}

// I-04: a replacement adapter is told what is owed. Yield not harvested before the swap stays yield.
#[test]
fn test_replacement_adapter_takes_its_opening_principal_as_an_argument() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    fixture.env.mock_all_auths();
    let deposit_amount = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);
    fixture.pool.add_yield(&fixture.usdc_token_id, &50_0000000);
    let unharvested = client.get_yield(&fixture.usdc_token_id);
    assert!(unharvested > 0);

    let replacement_id = fixture.env.register(
        BlendCapitalAdapter,
        (
            fixture.yield_controller.clone(),
            fixture.pool.address.clone(),
            fixture.blend_token_id.clone(),
            Address::generate(&fixture.env),
            fixture.usdc_token_id.clone(),
            client.get_principal(&fixture.usdc_token_id),
        ),
    );
    let replacement = LendingAdapterClient::new(&fixture.env, &replacement_id);
    assert_eq!(replacement.get_principal(&fixture.usdc_token_id), deposit_amount);
    assert_eq!(replacement.get_yield(&fixture.usdc_token_id), unharvested);

    // A member deposit right after the swap is principal, not yield.
    replacement.deposit(&fixture.user1, &fixture.usdc_token_id, &100_0000000);
    assert_eq!(replacement.get_principal(&fixture.usdc_token_id), deposit_amount + 100_0000000);
}

// L-06: an upgrade is announced with the new wasm hash.
#[test]
fn test_upgrade_emits_an_event() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();
    let hash = fixture.env.deployer().upload_contract_wasm(UPGRADE_WASM);
    fixture.adapter.upgrade(&hash);
    let id = fixture.adapter.address.clone();
    assert_eq!(
        fixture.env.events().all().filter_by_contract(&id),
        vec![&fixture.env, (id.clone(), (Symbol::new(&fixture.env, "upgrade"),).into_val(&fixture.env), hash.into_val(&fixture.env))]
    );
}

/// Any valid wasm will do as the upgrade target: the event is emitted by the code being replaced.
const UPGRADE_WASM: &[u8] = include_bytes!("../artifacts/pool.wasm");
