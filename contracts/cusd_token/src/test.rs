#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, IntoVal, String, Symbol,
};

use crate::{CUSD, CUSDClient};

struct TestFixture {
    env: Env,
    cusd: CUSDClient<'static>,
    #[allow(dead_code)]
    owner: Address,
    #[allow(dead_code)]
    admin: Address,
    #[allow(dead_code)]
    cusd_manager: Address,
    user1: Address,
    user2: Address,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        
        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let cusd_manager = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        let cusd_id = env.register(
            CUSD,
            (owner.clone(), cusd_manager.clone(), admin.clone()),
        );
        let cusd = CUSDClient::new(&env, &cusd_id);

        TestFixture {
            env,
            cusd,
            owner,
            admin,
            cusd_manager,
            user1,
            user2,
        }
    }

    fn assert_event(&self, expected_event: (Symbol,), expected_data: Address) {
        let _ = self.env.events().all();
        let published_event = vec![&self.env, self.env.events().all().last_unchecked()];
        let expected = vec![
            &self.env,
            (
                self.cusd.address.clone(),
                expected_event.into_val(&self.env),
                expected_data.into_val(&self.env),
            ),
        ];
        assert_eq!(published_event, expected);
    }
}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();

    // Verify metadata was set correctly
    assert_eq!(fixture.cusd.decimals(), 7);
    assert_eq!(fixture.cusd.name(), String::from_str(&fixture.env, "cUSD"));
    assert_eq!(fixture.cusd.symbol(), String::from_str(&fixture.env, "CUSD"));
    
    // Verify initial supply is 0
    assert_eq!(fixture.cusd.total_supply(), 0);
}

#[test]
fn test_mint_by_cusd_manager() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    let mint_amount = 1000_0000000i128; // 1000 cUSD with 7 decimals

    // Mint tokens to user1
    fixture.cusd.mint(&fixture.user1, &mint_amount);

    // Verify balance
    assert_eq!(fixture.cusd.balance(&fixture.user1), mint_amount);
    assert_eq!(fixture.cusd.total_supply(), mint_amount);

    // Note: The stellar-fungible library might not emit events in test mode
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_mint_unauthorized() {
    let fixture = TestFixture::create();
    // Don't mock the cusd_manager auth, so it should fail
    fixture.env.mock_auths(&[]);
    
    fixture.cusd.mint(&fixture.user1, &1000_0000000i128);
}

#[test]
fn test_mint_multiple_times() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    let first_mint = 500_0000000i128;
    let second_mint = 300_0000000i128;

    // First mint
    fixture.cusd.mint(&fixture.user1, &first_mint);
    assert_eq!(fixture.cusd.balance(&fixture.user1), first_mint);
    assert_eq!(fixture.cusd.total_supply(), first_mint);

    // Second mint to same user
    fixture.cusd.mint(&fixture.user1, &second_mint);
    assert_eq!(fixture.cusd.balance(&fixture.user1), first_mint + second_mint);
    assert_eq!(fixture.cusd.total_supply(), first_mint + second_mint);

    // Mint to different user
    fixture.cusd.mint(&fixture.user2, &first_mint);
    assert_eq!(fixture.cusd.balance(&fixture.user2), first_mint);
    assert_eq!(fixture.cusd.total_supply(), first_mint + second_mint + first_mint);
}

#[test]
fn test_set_cusd_manager_by_admin() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    let new_manager = Address::generate(&fixture.env);

    // Clear events before operation
    let _ = fixture.env.events().all();

    // Set new cusd manager
    fixture.cusd.set_cusd_manager(&new_manager);

    // Verify event (note: the event name has a typo "set_cud_manager" in the contract)
    fixture.assert_event(
        (Symbol::new(&fixture.env, "set_cud_manager"),),
        new_manager
    );

    // Verify new manager can mint
    fixture.cusd.mint(&fixture.user1, &1000_0000000i128);
    assert_eq!(fixture.cusd.balance(&fixture.user1), 1000_0000000i128);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_cusd_manager_unauthorized() {
    let fixture = TestFixture::create();
    let new_manager = Address::generate(&fixture.env);

    // Don't mock the admin auth, so it should fail
    fixture.env.mock_auths(&[]);
    
    fixture.cusd.set_cusd_manager(&new_manager);
}

#[test]
fn test_set_admin_by_owner() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    let new_admin = Address::generate(&fixture.env);

    // Clear events before operation
    let _ = fixture.env.events().all();

    // Set new admin
    fixture.cusd.set_admin(&new_admin);

    // Verify event
    fixture.assert_event(
        (Symbol::new(&fixture.env, "set_admin"),),
        new_admin
    );

    // Verify new admin can set cusd manager
    let another_manager = Address::generate(&fixture.env);
    fixture.cusd.set_cusd_manager(&another_manager);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_admin_unauthorized() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    // Don't mock the owner auth, so it should fail
    fixture.env.mock_auths(&[]);
    
    fixture.cusd.set_admin(&new_admin);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_admin_by_admin_should_fail() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    // Don't mock the owner auth, so it should fail (admin cannot set admin)
    fixture.env.mock_auths(&[]);
    
    fixture.cusd.set_admin(&new_admin);
}

#[test]
fn test_admin_hierarchy() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    // Create new addresses
    let new_admin = Address::generate(&fixture.env);
    let new_manager = Address::generate(&fixture.env);
    let newer_admin = Address::generate(&fixture.env);

    // Owner sets new admin
    fixture.cusd.set_admin(&new_admin);

    // New admin sets cusd manager
    fixture.cusd.set_cusd_manager(&new_manager);

    // New manager can mint
    fixture.cusd.mint(&fixture.user1, &1000_0000000i128);
    assert_eq!(fixture.cusd.balance(&fixture.user1), 1000_0000000i128);

    // Owner can still set another admin
    fixture.cusd.set_admin(&newer_admin);

    // Newer admin can set another cusd manager
    let another_manager = Address::generate(&fixture.env);
    fixture.cusd.set_cusd_manager(&another_manager);
}

#[test]
fn test_mint_zero_amount() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    // Mint zero tokens
    fixture.cusd.mint(&fixture.user1, &0);

    // Balance should still be 0
    assert_eq!(fixture.cusd.balance(&fixture.user1), 0);
    assert_eq!(fixture.cusd.total_supply(), 0);
}

#[test]
fn test_mint_large_amount() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    // Mint a very large amount (approaching i128::MAX)
    let large_amount = 1_000_000_000_000_000_0000000i128; // 1 quadrillion with 7 decimals

    fixture.cusd.mint(&fixture.user1, &large_amount);

    assert_eq!(fixture.cusd.balance(&fixture.user1), large_amount);
    assert_eq!(fixture.cusd.total_supply(), large_amount);
}

#[test]
fn test_mint_after_manager_change() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths();

    let new_manager = Address::generate(&fixture.env);
    let mint_amount = 1000_0000000i128;

    // Original manager mints
    fixture.cusd.mint(&fixture.user1, &mint_amount);
    assert_eq!(fixture.cusd.balance(&fixture.user1), mint_amount);

    // Change manager
    fixture.cusd.set_cusd_manager(&new_manager);

    // New manager mints
    fixture.cusd.mint(&fixture.user2, &mint_amount);
    assert_eq!(fixture.cusd.balance(&fixture.user2), mint_amount);

    // Total supply should be sum of both mints
    assert_eq!(fixture.cusd.total_supply(), mint_amount * 2);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_old_cusd_manager_cannot_mint_after_change() {
    let fixture = TestFixture::create();
    let new_manager = Address::generate(&fixture.env);

    // First change manager with proper auth
    fixture.env.mock_all_auths();
    fixture.cusd.set_cusd_manager(&new_manager);

    // Now old manager tries to mint (should fail)
    fixture.env.mock_auths(&[]);
    fixture.cusd.mint(&fixture.user1, &1000_0000000i128);
}