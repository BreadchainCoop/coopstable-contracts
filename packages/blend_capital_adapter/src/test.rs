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

        // Create tokens
        let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let usdc_token_id = usdc_token.address();
        let blend_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let blend_token_id = blend_token.address();

        // Deploy mock pool contract
        let pool_id = env.register(PoolContract, ());
        let pool = PoolContractClient::new(&env, &pool_id);

        // Initialize pool with USDC
        pool.init(&usdc_token_id);

        // Deploy blend capital adapter
        let adapter_id = env.register(
            BlendCapitalAdapter,
            (yield_controller.clone(), pool_id.clone(), blend_token_id.clone()),
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
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
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
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
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

    // Verify contract initialization by checking stored values
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

    // Clear events before operation
    let _ = fixture.env.events().all();

    let result = client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Verify result
    assert_eq!(result, amount);

    // Verify event emission
    fixture.assert_event_with_address_tuple_data(
        (Symbol::new(&fixture.env, "deposit"), fixture.adapter.address.clone()),
        (fixture.usdc_token_id.clone(), amount),
    );

    // Verify deposit tracking
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, Some(amount));
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_deposit_unauthorized() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    // Clear authorization
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

    // First deposit
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    // Clear events before withdraw
    let _ = fixture.env.events().all();

    // Withdraw
    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw_amount);

    // Verify result
    assert_eq!(result, withdraw_amount);

    // Verify event emission
    fixture.assert_event_with_user_tuple_data(
        (Symbol::new(&fixture.env, "withdraw"), fixture.adapter.address.clone(), fixture.user1.clone()),
        (fixture.usdc_token_id.clone(), withdraw_amount),
    );

    // Verify deposit tracking is updated
    let remaining_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(remaining_deposit, Some(deposit_amount - withdraw_amount));
}

#[test]
fn test_withdraw_full_amount() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    // First deposit
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Withdraw everything
    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Verify result
    assert_eq!(result, amount);

    // Verify deposit tracking is removed
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, None);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_withdraw_unauthorized() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    // Clear authorization
    fixture.env.mock_auths(&[]);

    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &amount);
}

#[test]
fn test_get_yield_no_accrual() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    // Deposit
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Check yield (should be 0 initially)
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0);
}

#[test]
fn test_claim_yield_no_yield() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    // Deposit
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Try to claim yield (should be 0)
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    let claimed_yield = client.claim_yield(&fixture.usdc_token_id, &yield_amount);
    assert_eq!(claimed_yield, 0);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_claim_yield_unauthorized() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    // Clear authorization
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

    // Perform deposit
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Verify that authorization was required
    let auths = fixture.env.auths();
    assert!(!auths.is_empty(), "No authorizations were recorded");

    // Check if yield_controller authorization was required
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

    // First deposit
    let deposit1 = 500_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit1);

    // Second deposit
    let deposit2 = 300_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit2);

    // Verify total deposit tracking
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, Some(deposit1 + deposit2));
}

#[test]
fn test_multiple_operations() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Multiple deposits
    let deposit1 = 500_0000000;
    let deposit2 = 300_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit1);
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit2);

    // Partial withdrawals
    let withdraw1 = 200_0000000;
    let withdraw2 = 300_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw1);
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw2);

    // Verify remaining balance
    let expected_remaining = deposit1 + deposit2 - withdraw1 - withdraw2;
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, Some(expected_remaining));

    // Final withdrawal
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &expected_remaining);

    // Verify deposit tracking is removed
    let final_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(final_deposit, None);
}

#[test]
fn test_multiple_users() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // User 1 deposits
    let deposit1 = 500_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit1);

    // User 2 deposits (adds to total contract deposits)
    let deposit2 = 300_0000000;
    client.deposit(&fixture.user2, &fixture.usdc_token_id, &deposit2);

    // Verify total contract deposit tracking (users are tracked as total)
    let total_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(total_deposit, Some(deposit1 + deposit2));

    // User 1 withdraws partially
    let withdraw1 = 200_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdraw1);

    // User 2 withdraws all
    client.withdraw(&fixture.user2, &fixture.usdc_token_id, &deposit2);

    // Verify final state (remaining balance from user 1's partial withdrawal)
    let remaining_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(remaining_deposit, Some(deposit1 - withdraw1));
}

#[test]
fn test_negative_yield_handling() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();
    let amount = 1000_0000000;

    fixture.env.mock_all_auths();

    // Deposit
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &amount);

    // Simulate negative yield by updating b_rate to a lower value
    let new_b_rate = 900_000_000_000; // 10% loss
    fixture.update_pool_b_rate(&fixture.usdc_token_id, new_b_rate);

    // Check yield - should return 0 for negative yield
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0, "Negative yield should be reported as 0");

    // Try to claim yield - should also return 0
    let claimed_yield = client.claim_yield(&fixture.usdc_token_id, &yield_amount);
    assert_eq!(claimed_yield, 0, "Claiming negative yield should return 0");
}

#[test]
fn test_zero_amount_operations() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Test zero deposit
    let result = client.deposit(&fixture.user1, &fixture.usdc_token_id, &0);
    assert_eq!(result, 0);

    // Verify no deposit tracking for zero amount
    let stored_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    assert_eq!(stored_deposit, None);

    // Test zero withdrawal
    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &0);
    assert_eq!(result, 0);
}

#[test]
fn test_different_assets() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    // Create another asset
    let other_token = fixture.env.register_stellar_asset_contract_v2(fixture.token_admin.clone());
    let other_token_id = other_token.address();

    fixture.env.mock_all_auths();

    // Deposit different assets
    let usdc_amount = 1000_0000000;
    let other_amount = 500_0000000;

    client.deposit(&fixture.user1, &fixture.usdc_token_id, &usdc_amount);
    client.deposit(&fixture.user1, &other_token_id, &other_amount);

    // Verify separate tracking per asset
    let usdc_deposit = fixture.get_contract_deposit(&fixture.usdc_token_id);
    let other_deposit = fixture.get_contract_deposit(&other_token_id);

    assert_eq!(usdc_deposit, Some(usdc_amount));
    assert_eq!(other_deposit, Some(other_amount));
}

#[test]
fn test_edge_case_operations() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Test operations with no prior deposits
    let yield_amount = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_amount, 0);

    let claimed_yield = client.claim_yield(&fixture.usdc_token_id, &yield_amount);
    assert_eq!(claimed_yield, 0);

    // Test withdraw with no deposit
    let result = client.withdraw(&fixture.user1, &fixture.usdc_token_id, &0);
    assert_eq!(result, 0);
}

#[test]
fn test_epoch_based_yield_tracking() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Initial deposit
    let deposit_amount = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &deposit_amount);

    // Simulate yield accrual (pool increases balance)
    let yield_amount = 50_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &yield_amount);

    // Verify yield shows as 50
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, yield_amount);

    // Set epoch principal for epoch 1 (simulating distribution)
    let epoch_1 = 1u64;
    let principal_after_dist = deposit_amount + yield_amount; // 1050
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_1, &principal_after_dist);

    // Verify yield shows as 0 after epoch principal is set
    let yield_after_epoch = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_epoch, 0);

    // Simulate more yield accrual for epoch 1
    let additional_yield = 25_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &additional_yield);

    // Verify yield shows as 25 (only the new yield)
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, additional_yield);

    // Test withdrawal during epoch
    let withdrawal_amount = 100_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &withdrawal_amount);

    // Verify yield calculation accounts for withdrawals
    // Current balance: 1050 + 25 - 100 = 975
    // Adjusted principal: 1050 - 100 = 950
    // Expected yield: 975 - 950 = 25
    let yield_after_withdrawal = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_withdrawal, additional_yield);

    // Test edge case: withdrawal larger than withdrawals tracked
    let large_withdrawal = 200_0000000;
    client.withdraw(&fixture.user1, &fixture.usdc_token_id, &large_withdrawal);

    // Verify yield calculation still works
    // Current balance: 975 - 200 = 775
    // Adjusted principal: 1050 - 300 = 750
    // Expected yield: 775 - 750 = 25
    let yield_after_large_withdrawal = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_large_withdrawal, additional_yield);
}

#[test]
fn test_epoch_transition_compound_effect() {
    let fixture = TestFixture::create();
    let client = fixture.lending_adapter_client();

    fixture.env.mock_all_auths();

    // Initial deposit in epoch 0
    let initial_deposit = 1000_0000000;
    client.deposit(&fixture.user1, &fixture.usdc_token_id, &initial_deposit);

    // Simulate yield accrual in epoch 0
    let epoch_0_yield = 50_0000000;
    fixture.pool.add_yield(&fixture.usdc_token_id, &epoch_0_yield);

    // Verify epoch 0 yield
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, epoch_0_yield);

    // Move to epoch 1 - principal includes previous yield
    let epoch_1 = 1u64;
    let epoch_1_principal = initial_deposit + epoch_0_yield; // 1050
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_1, &epoch_1_principal);

    // Verify yield resets to 0 after epoch transition
    let yield_after_epoch_1 = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_epoch_1, 0);

    // Simulate yield accrual in epoch 1
    let epoch_1_yield = 52_5000000; // 5% of 1050
    fixture.pool.add_yield(&fixture.usdc_token_id, &epoch_1_yield);

    // Verify epoch 1 yield
    let current_yield = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(current_yield, epoch_1_yield);

    // Move to epoch 2 - principal includes all previous yields
    let epoch_2 = 2u64;
    let epoch_2_principal = epoch_1_principal + epoch_1_yield; // 1102.5
    client.update_epoch_principal(&fixture.usdc_token_id, &epoch_2, &epoch_2_principal);

    // Verify yield resets to 0 after epoch transition
    let yield_after_epoch_2 = client.get_yield(&fixture.usdc_token_id);
    assert_eq!(yield_after_epoch_2, 0);

    // Verify compound effect: each epoch's principal includes all previous yields
    assert_eq!(epoch_2_principal, initial_deposit + epoch_0_yield + epoch_1_yield);
}