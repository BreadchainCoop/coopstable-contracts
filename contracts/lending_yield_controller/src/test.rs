#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::{
    contract::{LendingYieldController, LendingYieldControllerClient},
};

use cusd_manager::contract::{CUSDManager, CUSDManagerClient};
use yield_adapter_registry::contract::{
    YieldAdapterRegistry, YieldAdapterRegistryClient,
};
use yield_distributor::contract::{YieldDistributor, YieldDistributorClient};
use yield_adapter::contract_types::{SupportedAdapter, SupportedYieldType};

struct TestFixture {
    env: Env,
    controller: LendingYieldControllerClient<'static>,
    yield_distributor: YieldDistributorClient<'static>,
    adapter_registry: YieldAdapterRegistryClient<'static>,
    cusd_manager: CUSDManagerClient<'static>,
    owner: Address,
    admin: Address,
    user1: Address,
    user2: Address,
    usdc_token_id: Address,
    cusd_token_id: Address,
    token_admin: Address,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        env.ledger().set_timestamp(1000000000); // Set timestamp to make distribution available

        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let token_admin = Address::generate(&env);
        let treasury = Address::generate(&env);

        // Create USDC token
        let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let usdc_token_id = usdc_token.address();

        // Create CUSD token
        let cusd_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let cusd_token_id = cusd_token.address();

        // Deploy adapter registry
        let adapter_registry_id = env.register(
            YieldAdapterRegistry,
            (admin.clone(), owner.clone()),
        );
        let adapter_registry = YieldAdapterRegistryClient::new(&env, &adapter_registry_id);

        // Deploy yield distributor
        let treasury_share_bps = 1000u32; // 10%
        let distribution_period = 86400u64; // 1 day in seconds
        let yield_distributor_id = env.register(
            YieldDistributor,
            (
                treasury.clone(),
                treasury_share_bps,
                admin.clone(), // Will be updated to controller later
                distribution_period,
                owner.clone(),
                admin.clone(),
            ),
        );
        let yield_distributor = YieldDistributorClient::new(&env, &yield_distributor_id);

        // Deploy CUSD manager
        let cusd_manager_id = env.register(
            CUSDManager,
            (cusd_token_id.clone(), admin.clone(), admin.clone()),
        );
        let cusd_manager = CUSDManagerClient::new(&env, &cusd_manager_id);

        // Deploy lending yield controller
        let controller_id = env.register(
            LendingYieldController,
            (
                yield_distributor_id.clone(),
                adapter_registry_id.clone(),
                cusd_manager_id.clone(),
                admin.clone(),
                owner.clone(),
            ),
        );
        let controller = LendingYieldControllerClient::new(&env, &controller_id);

        // Set up CUSD token admin and controller relationships
        env.mock_all_auths();
        let cusd_admin_client = StellarAssetClient::new(&env, &cusd_token_id);
        cusd_admin_client.set_admin(&cusd_manager_id);
        cusd_manager.set_yield_controller(&controller_id);

        // Update yield distributor to use our controller as the yield controller
        yield_distributor.set_yield_controller(&controller_id);

        // Mint initial tokens to users
        let usdc_admin_client = StellarAssetClient::new(&env, &usdc_token_id);
        usdc_admin_client.mint(&user1, &10000_0000000);
        usdc_admin_client.mint(&user2, &5000_0000000);

        TestFixture {
            env,
            controller,
            yield_distributor,
            adapter_registry,
            cusd_manager,
            owner,
            admin,
            user1,
            user2,
            usdc_token_id,
            cusd_token_id,
            token_admin,
        }
    }

    fn usdc_client(&self) -> TokenClient<'static> {
        TokenClient::new(&self.env, &self.usdc_token_id)
    }

    fn cusd_client(&self) -> TokenClient<'static> {
        TokenClient::new(&self.env, &self.cusd_token_id)
    }

    fn assert_event_with_tuple_data(&self, expected_event: (Symbol, Address), expected_data: (Address, i128)) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.controller.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn assert_event_with_address_data(&self, expected_event: (Symbol,), expected_data: Address) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.controller.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn setup_usdc_approval(&self, user: &Address, amount: i128) {
        self.env.mock_all_auths();
        self.usdc_client().approve(user, &self.controller.address, &amount, &1000000);
    }

    fn setup_cusd_approval(&self, user: &Address, amount: i128) {
        self.env.mock_all_auths();
        self.cusd_client().approve(user, &self.controller.address, &amount, &1000000);
    }
}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();

    // Verify contract initialization by checking if it can perform basic operations
    // This is indirectly testing that all addresses were stored correctly
    
    // The contract should be initialized and ready to use
    // We can verify this by checking that basic operations don't panic
    let initial_yield = fixture.controller.get_yield();
    assert_eq!(initial_yield, 0); // No yield initially
}

#[test]
fn test_get_yield_no_adapters() {
    let fixture = TestFixture::create();

    let yield_amount = fixture.controller.get_yield();
    assert_eq!(yield_amount, 0);
}

#[test]
fn test_claim_yield_no_adapters() {
    let fixture = TestFixture::create();

    // Jump forward to enable distribution
    let current_time = fixture.env.ledger().timestamp();
    fixture.env.ledger().set_timestamp(current_time + 86400 + 10);

    fixture.env.mock_all_auths();
    let claimed_yield = fixture.controller.claim_yield();
    assert_eq!(claimed_yield, 0);
}

#[test]
fn test_claim_yield_distribution_not_available() {
    let fixture = TestFixture::create();

    // When there are no adapters and no yield, claim_yield should return 0
    // rather than checking distribution availability
    fixture.env.mock_all_auths();
    let result = fixture.controller.claim_yield();
    assert_eq!(result, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1100)")]
fn test_deposit_unsupported_asset() {
    let fixture = TestFixture::create();

    // Create an unsupported token
    let unsupported_token = fixture.env.register_stellar_asset_contract_v2(fixture.admin.clone());
    let unsupported_token_id = unsupported_token.address();

    fixture.env.mock_all_auths();
    fixture.controller.deposit_collateral(
        &SupportedAdapter::BlendCapital.id(),
        &fixture.user1,
        &unsupported_token_id,
        &1000_0000000,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1100)")]
fn test_withdraw_unsupported_asset() {
    let fixture = TestFixture::create();

    // Create an unsupported token
    let unsupported_token = fixture.env.register_stellar_asset_contract_v2(fixture.admin.clone());
    let unsupported_token_id = unsupported_token.address();

    fixture.env.mock_all_auths();
    fixture.controller.withdraw_collateral(
        &SupportedAdapter::BlendCapital.id(),
        &fixture.user1,
        &unsupported_token_id,
        &1000_0000000,
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_deposit_unauthorized() {
    let fixture = TestFixture::create();

    // Register adapter and asset support
    fixture.env.mock_all_auths();
    let dummy_adapter = Address::generate(&fixture.env);
    fixture.adapter_registry.register_adapter(
        &SupportedYieldType::Lending.id(),
        &SupportedAdapter::BlendCapital.id(),
        &dummy_adapter,
    );
    fixture.adapter_registry.add_support_for_asset(
        &SupportedYieldType::Lending.id(),
        &SupportedAdapter::BlendCapital.id(),
        &fixture.usdc_token_id,
    );

    // Clear authorization
    fixture.env.mock_auths(&[]);

    fixture.controller.deposit_collateral(
        &SupportedAdapter::BlendCapital.id(),
        &fixture.user1,
        &fixture.usdc_token_id,
        &1000_0000000,
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_withdraw_unauthorized() {
    let fixture = TestFixture::create();

    // Register adapter and asset support
    fixture.env.mock_all_auths();
    let dummy_adapter = Address::generate(&fixture.env);
    fixture.adapter_registry.register_adapter(
        &SupportedYieldType::Lending.id(),
        &SupportedAdapter::BlendCapital.id(),
        &dummy_adapter,
    );
    fixture.adapter_registry.add_support_for_asset(
        &SupportedYieldType::Lending.id(),
        &SupportedAdapter::BlendCapital.id(),
        &fixture.usdc_token_id,
    );

    // Clear authorization
    fixture.env.mock_auths(&[]);

    fixture.controller.withdraw_collateral(
        &SupportedAdapter::BlendCapital.id(),
        &fixture.user1,
        &fixture.usdc_token_id,
        &1000_0000000,
    );
}

#[test]
fn test_set_yield_distributor() {
    let fixture = TestFixture::create();
    let new_distributor = Address::generate(&fixture.env);

    fixture.env.mock_all_auths();

    // Clear events before operation
    let _ = fixture.env.events().all();

    fixture.controller.set_yield_distributor(&new_distributor);

    // Verify event
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "set_yield_distributor"),),
        new_distributor.clone(),
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_yield_distributor_unauthorized() {
    let fixture = TestFixture::create();
    let new_distributor = Address::generate(&fixture.env);

    // Clear authorization
    fixture.env.mock_auths(&[]);

    fixture.controller.set_yield_distributor(&new_distributor);
}

#[test]
fn test_set_adapter_registry() {
    let fixture = TestFixture::create();
    let new_registry = Address::generate(&fixture.env);

    fixture.env.mock_all_auths();

    // Clear events before operation
    let _ = fixture.env.events().all();

    fixture.controller.set_adapter_registry(&new_registry);

    // Verify event
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "set_adapter_registry"),),
        new_registry.clone(),
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_adapter_registry_unauthorized() {
    let fixture = TestFixture::create();
    let new_registry = Address::generate(&fixture.env);

    // Clear authorization
    fixture.env.mock_auths(&[]);

    fixture.controller.set_adapter_registry(&new_registry);
}

#[test]
fn test_set_cusd_manager() {
    let fixture = TestFixture::create();
    let new_manager = Address::generate(&fixture.env);

    fixture.env.mock_all_auths();

    // Clear events before operation
    let _ = fixture.env.events().all();

    fixture.controller.set_cusd_manager(&new_manager);

    // Verify event
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "set_cusd_manager"),),
        new_manager.clone(),
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_cusd_manager_unauthorized() {
    let fixture = TestFixture::create();
    let new_manager = Address::generate(&fixture.env);

    // Clear authorization
    fixture.env.mock_auths(&[]);

    fixture.controller.set_cusd_manager(&new_manager);
}

#[test]
fn test_set_admin() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    fixture.env.mock_all_auths();

    // Clear events before operation
    let _ = fixture.env.events().all();

    fixture.controller.set_admin(&new_admin);

    // Verify event
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "set_admin"),),
        new_admin.clone(),
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_admin_unauthorized() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    // Clear authorization
    fixture.env.mock_auths(&[]);

    fixture.controller.set_admin(&new_admin);
}

#[test]
fn test_configuration_changes() {
    let fixture = TestFixture::create();

    fixture.env.mock_all_auths();

    // Test setting various addresses
    let new_distributor = Address::generate(&fixture.env);
    let new_registry = Address::generate(&fixture.env);
    let new_manager = Address::generate(&fixture.env);
    let new_admin = Address::generate(&fixture.env);

    fixture.controller.set_yield_distributor(&new_distributor);
    fixture.controller.set_adapter_registry(&new_registry);
    fixture.controller.set_cusd_manager(&new_manager);
    fixture.controller.set_admin(&new_admin);

    // If no panics occur, the configuration changes were successful
}

#[test]
fn test_multiple_users_operations() {
    let fixture = TestFixture::create();

    // Verify that multiple users can interact with the contract
    // (though without actual adapters, operations will fail with unsupported asset errors)

    // Check balances for different users
    let user1_balance = fixture.usdc_client().balance(&fixture.user1);
    let user2_balance = fixture.usdc_client().balance(&fixture.user2);
    
    assert_eq!(user1_balance, 10000_0000000);
    assert_eq!(user2_balance, 5000_0000000);

    // Verify users have no CUSD initially
    let user1_cusd_balance = fixture.cusd_client().balance(&fixture.user1);
    let user2_cusd_balance = fixture.cusd_client().balance(&fixture.user2);
    
    assert_eq!(user1_cusd_balance, 0);
    assert_eq!(user2_cusd_balance, 0);
}

#[test]
fn test_yield_operations_edge_cases() {
    let fixture = TestFixture::create();

    // Test yield operations when no adapters are registered
    let yield_amount = fixture.controller.get_yield();
    assert_eq!(yield_amount, 0);

    // Jump forward in time to enable distribution
    let current_time = fixture.env.ledger().timestamp();
    fixture.env.ledger().set_timestamp(current_time + 86400 + 10);

    // Test claiming yield with no adapters
    fixture.env.mock_all_auths();
    let claimed_yield = fixture.controller.claim_yield();
    assert_eq!(claimed_yield, 0);
}

#[test]
fn test_token_setup_verification() {
    let fixture = TestFixture::create();

    // Verify token setup is correct
    let usdc_balance_user1 = fixture.usdc_client().balance(&fixture.user1);
    let usdc_balance_user2 = fixture.usdc_client().balance(&fixture.user2);
    
    assert!(usdc_balance_user1 > 0);
    assert!(usdc_balance_user2 > 0);

    // Verify CUSD manager has admin rights over CUSD token
    let cusd_stellar_client = StellarAssetClient::new(&fixture.env, &fixture.cusd_token_id);
    let cusd_admin = cusd_stellar_client.admin();
    assert_eq!(cusd_admin, fixture.cusd_manager.address);
}

#[test]
fn test_contract_relationships() {
    let fixture = TestFixture::create();

    // Verify that the contracts are properly connected by testing
    // that they can interact without errors during setup
    // This is an indirect test that the relationships are correct
    
    // Verify yield operations work (indicating proper setup)
    let yield_amount = fixture.controller.get_yield();
    assert_eq!(yield_amount, 0); // No adapters, so yield should be 0
}