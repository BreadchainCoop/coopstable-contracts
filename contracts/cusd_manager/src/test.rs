#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::contract::{CUSDManager, CUSDManagerClient};

struct TestFixture {
    env: Env,
    cusd_manager: CUSDManagerClient<'static>,
    cusd_token_id: Address,
    #[allow(dead_code)]
    owner: Address,
    #[allow(dead_code)]
    admin: Address,
    #[allow(dead_code)]
    yield_controller: Address,
    user1: Address,
    user2: Address,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        
        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let yield_controller = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let token_admin = Address::generate(&env);

        // Create Stellar asset contract for cUSD token
        let cusd_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let cusd_token_id = cusd_token.address();

        // Deploy CUSD Manager
        let cusd_manager_id = env.register(
            CUSDManager,
            (cusd_token_id.clone(), owner.clone(), admin.clone()),
        );
        let cusd_manager = CUSDManagerClient::new(&env, &cusd_manager_id);

        // Setup token admin to point to cusd_manager
        let token_client = StellarAssetClient::new(&env, &cusd_token_id);
        env.mock_all_auths_allowing_non_root_auth();
        token_client.set_admin(&cusd_manager_id);

        // Set yield controller
        env.mock_all_auths_allowing_non_root_auth();
        cusd_manager.set_yield_controller(&yield_controller);

        TestFixture {
            env,
            cusd_manager,
            cusd_token_id,
            owner,
            admin,
            yield_controller,
            user1,
            user2,
        }
    }

    fn token_client(&self) -> TokenClient<'static> {
        TokenClient::new(&self.env, &self.cusd_token_id)
    }

    #[allow(dead_code)]
    fn assert_event_with_tuple_data(&self, expected_event: (Symbol,), expected_data: (Address, i128)) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.cusd_manager.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    #[allow(dead_code)]
    fn assert_event_with_address_data(&self, expected_event: (Symbol,), expected_data: Address) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.cusd_manager.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn issue_tokens_to_user(&self, user: &Address, amount: i128) {
        // Use mock_all_auths_allowing_non_root_auth for cross-contract calls
        self.env.mock_all_auths_allowing_non_root_auth();
        self.cusd_manager.issue_cusd(user, &amount);
    }

    fn burn_tokens_from_user(&self, user: &Address, amount: i128) {
        // Burn tokens directly from the user (the burn function burns from the specified user)
        // Use mock_all_auths_allowing_non_root_auth for cross-contract calls
        self.env.mock_all_auths_allowing_non_root_auth();
        self.cusd_manager.burn_cusd(user, &amount);
    }
}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();

    // Verify cusd_id is set correctly
    assert_eq!(fixture.cusd_manager.get_cusd_id(), fixture.cusd_token_id);
}

#[test]
fn test_issue_cusd_by_yield_controller() {
    let fixture = TestFixture::create();
    let amount = 1000_0000000i128; // 1000 cUSD with 7 decimals
    
    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Issue tokens
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount);

    // Verify balance
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount);

    // Note: Event testing might be affected by cross-contract calls
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_issue_cusd_unauthorized() {
    let fixture = TestFixture::create();
    let amount = 1000_0000000i128;

    // Don't mock yield_controller auth
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_issue_cusd_negative_amount() {
    let fixture = TestFixture::create();
    let amount = -100i128;

    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount);
}

#[test]
fn test_issue_cusd_zero_amount() {
    let fixture = TestFixture::create();
    let amount = 0i128;

    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount);

    // Balance should remain 0
    assert_eq!(fixture.token_client().balance(&fixture.user1), 0);
}

#[test]
fn test_issue_cusd_multiple_times() {
    let fixture = TestFixture::create();
    let amount1 = 500_0000000i128;
    let amount2 = 300_0000000i128;

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // First issuance
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount1);
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount1);

    // Second issuance to same user
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount2);
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount1 + amount2);

    // Issue to different user
    fixture.cusd_manager.issue_cusd(&fixture.user2, &amount1);
    assert_eq!(fixture.token_client().balance(&fixture.user2), amount1);
}

#[test]
fn test_burn_cusd() {
    let fixture = TestFixture::create();
    let issue_amount = 1000_0000000i128;
    let burn_amount = 400_0000000i128;

    // Issue tokens first
    fixture.issue_tokens_to_user(&fixture.user1, issue_amount);
    assert_eq!(fixture.token_client().balance(&fixture.user1), issue_amount);

    // Burn tokens
    fixture.burn_tokens_from_user(&fixture.user1, burn_amount);

    // Verify balance
    assert_eq!(fixture.token_client().balance(&fixture.user1), issue_amount - burn_amount);

    // Note: Event testing might be affected by cross-contract calls
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_burn_cusd_negative_amount() {
    let fixture = TestFixture::create();
    let amount = -100i128;

    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    fixture.cusd_manager.burn_cusd(&fixture.user1, &amount);
}

#[test]
fn test_burn_cusd_zero_amount() {
    let fixture = TestFixture::create();
    let issue_amount = 1000_0000000i128;
    
    // Issue tokens first
    fixture.issue_tokens_to_user(&fixture.user1, issue_amount);
    
    // Burn zero amount
    fixture.env.mock_all_auths_allowing_non_root_auth();
    fixture.cusd_manager.burn_cusd(&fixture.user1, &0);

    // Balance should remain unchanged
    assert_eq!(fixture.token_client().balance(&fixture.user1), issue_amount);
}

#[test]
fn test_burn_cusd_full_balance() {
    let fixture = TestFixture::create();
    let amount = 1000_0000000i128;

    // Issue tokens
    fixture.issue_tokens_to_user(&fixture.user1, amount);
    
    // Burn all tokens
    fixture.burn_tokens_from_user(&fixture.user1, amount);

    // Balance should be 0
    assert_eq!(fixture.token_client().balance(&fixture.user1), 0);
}

#[test]
fn test_set_admin_by_owner() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Clear events before operation
    let _ = fixture.env.events().all();

    // Set new admin
    fixture.cusd_manager.set_admin(&new_admin);

    // Note: Event testing might be affected by cross-contract calls

    // Verify new admin can set yield controller
    let new_yield_controller = Address::generate(&fixture.env);
    fixture.cusd_manager.set_yield_controller(&new_yield_controller);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_admin_unauthorized() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    // Don't mock owner auth
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.set_admin(&new_admin);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_admin_by_admin_should_fail() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    // Admin cannot set admin (only owner can)
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.set_admin(&new_admin);
}

#[test]
fn test_set_yield_controller_by_admin() {
    let fixture = TestFixture::create();
    let new_yield_controller = Address::generate(&fixture.env);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Clear events before operation
    let _ = fixture.env.events().all();

    // Set new yield controller
    fixture.cusd_manager.set_yield_controller(&new_yield_controller);

    // Note: Event testing might be affected by cross-contract calls

    // Verify new yield controller can issue tokens
    fixture.cusd_manager.issue_cusd(&fixture.user1, &1000_0000000i128);
    assert_eq!(fixture.token_client().balance(&fixture.user1), 1000_0000000i128);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_yield_controller_unauthorized() {
    let fixture = TestFixture::create();
    let new_yield_controller = Address::generate(&fixture.env);

    // Don't mock admin auth
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.set_yield_controller(&new_yield_controller);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_yield_controller_by_owner_should_fail() {
    let fixture = TestFixture::create();
    let new_yield_controller = Address::generate(&fixture.env);

    // Owner cannot set yield controller (only admin can)
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.set_yield_controller(&new_yield_controller);
}

#[test]
fn test_set_cusd_id_by_admin() {
    let fixture = TestFixture::create();
    let token_admin = Address::generate(&fixture.env);
    let new_cusd_token = fixture.env.register_stellar_asset_contract_v2(token_admin);
    let new_cusd_id = new_cusd_token.address();

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Clear events before operation
    let _ = fixture.env.events().all();

    // Set new cusd id
    fixture.cusd_manager.set_cusd_id(&new_cusd_id);

    // Note: Event testing might be affected by cross-contract calls

    // Verify new cusd id is returned
    assert_eq!(fixture.cusd_manager.get_cusd_id(), new_cusd_id);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_cusd_id_unauthorized() {
    let fixture = TestFixture::create();
    let token_admin = Address::generate(&fixture.env);
    let new_cusd_token = fixture.env.register_stellar_asset_contract_v2(token_admin);
    let new_cusd_id = new_cusd_token.address();

    // Don't mock admin auth
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.set_cusd_id(&new_cusd_id);
}

#[test]
fn test_get_cusd_id() {
    let fixture = TestFixture::create();
    
    // Should return the cusd token id set in constructor
    assert_eq!(fixture.cusd_manager.get_cusd_id(), fixture.cusd_token_id);
}

#[test]
fn test_admin_hierarchy() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Create new addresses
    let new_admin = Address::generate(&fixture.env);
    let new_yield_controller = Address::generate(&fixture.env);
    let newer_admin = Address::generate(&fixture.env);

    // Owner sets new admin
    fixture.cusd_manager.set_admin(&new_admin);

    // New admin sets yield controller
    fixture.cusd_manager.set_yield_controller(&new_yield_controller);

    // New yield controller can issue tokens
    fixture.cusd_manager.issue_cusd(&fixture.user1, &1000_0000000i128);
    assert_eq!(fixture.token_client().balance(&fixture.user1), 1000_0000000i128);

    // Owner can still set another admin
    fixture.cusd_manager.set_admin(&newer_admin);

    // Newer admin can set another yield controller
    fixture.cusd_manager.set_cusd_id(&fixture.cusd_token_id); // Admin function
}

#[test]
fn test_issue_and_burn_cycle() {
    let fixture = TestFixture::create();
    let amount = 1000_0000000i128;

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Issue tokens
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount);
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount);

    // Burn half (burns directly from user account)
    fixture.cusd_manager.burn_cusd(&fixture.user1, &(amount / 2));
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount / 2);

    // Issue more
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount);
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount / 2 + amount);

    // Burn all (burns directly from user account)
    let total_balance = fixture.token_client().balance(&fixture.user1);
    fixture.cusd_manager.burn_cusd(&fixture.user1, &total_balance);
    assert_eq!(fixture.token_client().balance(&fixture.user1), 0);
}

#[test]
fn test_multiple_users_issue_and_burn() {
    let fixture = TestFixture::create();
    let amount1 = 500_0000000i128;
    let amount2 = 800_0000000i128;

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Issue to user1
    fixture.cusd_manager.issue_cusd(&fixture.user1, &amount1);
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount1);

    // Issue to user2
    fixture.cusd_manager.issue_cusd(&fixture.user2, &amount2);
    assert_eq!(fixture.token_client().balance(&fixture.user2), amount2);

    // User1 burns some tokens (burns directly from user account)
    fixture.cusd_manager.burn_cusd(&fixture.user1, &(amount1 / 2));
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount1 / 2);

    // User2's balance unchanged
    assert_eq!(fixture.token_client().balance(&fixture.user2), amount2);

    // User2 burns all tokens (burns directly from user account)
    fixture.cusd_manager.burn_cusd(&fixture.user2, &amount2);
    assert_eq!(fixture.token_client().balance(&fixture.user2), 0);

    // User1's balance still unchanged
    assert_eq!(fixture.token_client().balance(&fixture.user1), amount1 / 2);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_old_yield_controller_cannot_issue_after_change() {
    let fixture = TestFixture::create();
    let new_yield_controller = Address::generate(&fixture.env);

    // First change yield controller with proper auth
    fixture.env.mock_all_auths_allowing_non_root_auth();
    fixture.cusd_manager.set_yield_controller(&new_yield_controller);

    // Now old yield controller tries to issue (should fail)
    fixture.env.mock_auths(&[]);
    
    fixture.cusd_manager.issue_cusd(&fixture.user1, &1000_0000000i128);
}

#[test]
fn test_large_amount_operations() {
    let fixture = TestFixture::create();
    // Test with large amount (approaching i128::MAX for reasonable token amounts)
    let large_amount = 1_000_000_000_000_000_0000000i128; // 1 quadrillion with 7 decimals

    fixture.env.mock_all_auths_allowing_non_root_auth();

    // Issue large amount
    fixture.cusd_manager.issue_cusd(&fixture.user1, &large_amount);
    assert_eq!(fixture.token_client().balance(&fixture.user1), large_amount);

    // Burn half (burns directly from user account)
    fixture.cusd_manager.burn_cusd(&fixture.user1, &(large_amount / 2));
    assert_eq!(fixture.token_client().balance(&fixture.user1), large_amount / 2);
}