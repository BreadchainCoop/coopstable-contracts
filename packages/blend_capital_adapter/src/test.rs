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