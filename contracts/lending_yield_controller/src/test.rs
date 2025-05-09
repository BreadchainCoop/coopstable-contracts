#![cfg(test)]
extern crate std;

use soroban_sdk::{
    testutils::{
        Address as _, 
        Events, 
        Ledger as _
    }, token::{StellarAssetClient, TokenClient}, vec, Address, Env, Symbol, IntoVal
};
use pretty_assertions::assert_eq;

use crate::contract::{
    LendingYieldController, LendingYieldControllerClient, LendingYieldControllerArgs
};
use yield_adapter::contract_types::{SupportedAdapter, SupportedYieldType};
use yield_adapter_registry::contract::{YieldAdapterRegistryClient, YieldAdapterRegistry, YieldAdapterRegistryArgs};
use yield_distributor::contract::{YieldDistributorClient, YieldDistributor, YieldDistributorArgs};
use cusd_manager::contract::{
    CUSDManagerClient,
    CUSDManager,
    CUSDManagerArgs
};

mod mock_adapter {
    use soroban_sdk::{contract, contractimpl, contracttype, token::TokenClient, Address, Env, IntoVal};
    
    #[derive(Clone)]
    #[contracttype]
    struct Yield {
        amount: i128,
        asset: Address
    }
    
    #[contract]
    pub struct MockLendingAdapter;

    #[contractimpl]
    impl MockLendingAdapter {
        pub fn set_mock_yield(e: &Env, user: Address, asset: Address, amount: i128) {
            let mock_yield = Yield {
                amount,
                asset
            };
            e.storage().instance().set(&user, &mock_yield);
        }

        pub fn deposit(
            e: &Env,
            user: Address,
            asset: Address,
            amount: i128
        ) -> i128 {
            // Return the deposited amount
            amount
        }
        
        pub fn withdraw(
            e: &Env,
            user: Address,
            asset: Address,
            amount: i128
        ) -> i128 {
            // Return the withdrawn amount
            amount
        }
        
        pub fn get_yield(
            e: &Env,
            user: Address,
            asset: Address
        ) -> i128 {
            let mock_yield: Option<Yield> = e.storage().instance().get(&user);
            match mock_yield {
                Some(yield_data) => yield_data.amount,
                None => 0
            }
        }

        pub fn claim_yield(
            e: &Env, 
            user: Address,
            asset: Address
        )   -> i128 {
            let yield_amount = Self::get_yield(e, user.clone(), asset.clone());

            let token_client = TokenClient::new(e, &asset);
            token_client.transfer(&e.current_contract_address(), &user, &yield_amount);

            yield_amount
        }
    }       
}

// Test fixture to simplify test setup and provide common utilities
struct TestFixture {
    env: Env,
    controller: LendingYieldControllerClient<'static>,
    yield_distributor: YieldDistributorClient<'static>,
    adapter_registry: YieldAdapterRegistryClient<'static>,
    cusd_manager: CUSDManagerClient<'static>,
    admin: Address,
    user: Address,
    treasury: Address,
    usdc_token: Address,
    usdc_client: TokenClient<'static>,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        
        // Create test addresses
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let treasury = Address::generate(&env);
        
        // Create USDC token
        let token_admin = Address::generate(&env);
        let usdc_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let usdc_token_id = usdc_token.address();
        let usdc_client = TokenClient::new(&env, &usdc_token_id);
        let usdc_admin_client = StellarAssetClient::new(&env, &usdc_token_id);
        
        // Initialize USDC with some balance for the test user
        env.mock_all_auths();
        StellarAssetClient::new(&env, &usdc_token_id).mint(&user, &10000_0000000);
        
        // Deploy adapter registry contract
        let adapter_registry_id = env.register(
            YieldAdapterRegistry,
            YieldAdapterRegistryArgs::__constructor(&admin)
        );
        let adapter_registry = YieldAdapterRegistryClient::new(&env, &adapter_registry_id);
        
        // Deploy yield distributor contract
        let treasury_share_bps: u32 = 1000; // 10%
        let distribution_period: u64 = 86400; // 1 day in seconds
        
        let yield_distributor_id = env.register(
            YieldDistributor,
            YieldDistributorArgs::__constructor(
                &admin,
                &treasury,
                &treasury_share_bps,
                &token_admin, // Temporarily use token_admin as yield controller
                &distribution_period
            )
        );
        let yield_distributor = YieldDistributorClient::new(&env, &yield_distributor_id);
        
        // Deploy CUSD token and manager
        let cusd_token = env.register_stellar_asset_contract_v2(token_admin.clone());
        let cusd_token_id = cusd_token.address();
        
        let cusd_manager_id = env.register(
            CUSDManager,
            CUSDManagerArgs::__constructor(&cusd_token_id, &admin, &admin)
        );
        let cusd_manager = CUSDManagerClient::new(&env, &cusd_manager_id);
        env.mock_all_auths();
        usdc_admin_client.set_admin(&cusd_manager_id);
        // env.mock_all_auths();
        // cusd_manager.set_cusd_issuer(&admin, &cusd_manager_id);

        // Deploy LendingYieldController
        let controller_id = env.register(
            LendingYieldController,
            LendingYieldControllerArgs::__constructor(
                &yield_distributor_id,
                &adapter_registry_id,
                &cusd_manager_id
            )
        );
        let controller = LendingYieldControllerClient::new(&env, &controller_id);
        cusd_manager.set_cusd_manager_admin(&admin, &controller_id);
        
        // Update yield distributor to use our controller as the yield controller
        env.mock_all_auths();
        env.as_contract(&yield_distributor_id, || {
            env.storage().instance().set(&Symbol::new(&env, "YC"), &controller_id);
        });
        
        Self {
            env,
            controller,
            yield_distributor,
            adapter_registry,
            cusd_manager,
            admin,
            user,
            treasury,
            usdc_token: usdc_token_id,
            usdc_client,
        }
    }
    
    // Helper to create and register a mock lending adapter
    fn create_mock_lending_adapter(&self, protocol: SupportedAdapter) -> mock_adapter::MockLendingAdapterClient<'static> {
        // Deploy the mock lending adapter contract
        let mock_adapter_id = self.env.register(
            mock_adapter::MockLendingAdapter,
            ()
        );
        
        // Register the adapter in the registry
        self.env.mock_all_auths();
        self.adapter_registry.register_adapter(
            &self.admin,
            &SupportedYieldType::Lending.id(),
            &protocol.id(),
            &mock_adapter_id
        );
        
        // Add support for USDC in this adapter
        self.adapter_registry.add_support_for_asset(
            &self.admin,
            &SupportedYieldType::Lending.id(),
            &protocol.id(),
            &self.usdc_token
        );
        
        mock_adapter::MockLendingAdapterClient::new(&self.env, &mock_adapter_id)
    }
    
    
    // Jump forward in time
    fn jump(&self, seconds: u64) {
        let current_time = self.env.ledger().timestamp();
        self.env.ledger().set_timestamp(current_time + seconds);
    }
    
    // Helper to setup approvals for transferring tokens
    fn approve_tokens(&self, from: &Address, amount: i128) {
        self.env.mock_all_auths();
        self.usdc_client.approve(from, &self.controller.address, &amount, &100000);
    }

    fn set_adapter_mock_yield(&self, adapter_id: &Address, user: &Address, asset: &Address, amount: i128) {
        self.env.mock_all_auths();
        mock_adapter::MockLendingAdapterClient::new(&self.env, &adapter_id).set_mock_yield(user, asset, &amount);
    }

}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();
    
    // Verify initialization by checking stored contract addresses
    fixture.env.as_contract(&fixture.controller.address, || {
        let adapter_registry = fixture.env.storage().instance().get::<Symbol, Address>(&Symbol::new(&fixture.env, "AR")).unwrap();
        let cusd_manager = fixture.env.storage().instance().get::<Symbol, Address>(&Symbol::new(&fixture.env, "CM")).unwrap();
        let yield_distributor = fixture.env.storage().instance().get::<Symbol, Address>(&Symbol::new(&fixture.env, "YD")).unwrap();
        
        assert_eq!(adapter_registry, fixture.adapter_registry.address);
        assert_eq!(cusd_manager, fixture.cusd_manager.address);
        assert_eq!(yield_distributor, fixture.yield_distributor.address);
    });
}

#[test]
fn test_deposit_collateral() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter
    let _ = fixture.create_mock_lending_adapter(SupportedAdapter::BlendCapital);
    
    // Setup token approval
    let deposit_amount = 1000_0000000;
    fixture.approve_tokens(&fixture.user, deposit_amount);
    
    // Deposit collateral
    fixture.env.mock_all_auths_allowing_non_root_auth();
    let result = fixture.controller.deposit_collateral(
        &SupportedAdapter::BlendCapital.id(),
        &fixture.user,
        &fixture.usdc_token,
        &deposit_amount
    );
    
    // Check for deposit event from adapter
    let expected_event = vec![&fixture.env, fixture.env.events().all().last_unchecked()];
    let control_event = vec![
        &fixture.env,
        (
            fixture.controller.address.clone(),
            (Symbol::new(&fixture.env, "deposit_collateral"), &fixture.user).into_val(&fixture.env),
            (
                fixture.usdc_token.clone(),
                &deposit_amount
            ).into_val(&fixture.env)
        )
    ];
    assert_eq!(expected_event, control_event);
    
    // Verify result
    assert_eq!(result, deposit_amount);
    
    // Check CUSD issuance
    let cusd_id = fixture.cusd_manager.get_cusd_id();
    let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
    let cusd_balance = cusd_client.balance(&fixture.user);
    assert_eq!(cusd_balance, deposit_amount, "CUSD tokens not issued correctly");
}

#[test]
fn test_withdraw_collateral() {
    let fixture = TestFixture::create();
    fixture.env.mock_all_auths_allowing_non_root_auth();
    let _ = fixture.create_mock_lending_adapter(SupportedAdapter::BlendCapital);
    
    // Create a mock lending adapter
    let protocol = SupportedAdapter::BlendCapital;
    
    // First deposit collateral
    let deposit_amount = 1000_0000000;
    fixture.approve_tokens(&fixture.user, deposit_amount);
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    fixture.controller.deposit_collateral(
        &protocol.id(),
        &fixture.user,
        &fixture.usdc_token,
        &deposit_amount
    );
    
    let cusd_id = fixture.cusd_manager.get_cusd_id();
    let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
    // Now withdraw half of the collateral
    let withdraw_amount = 500_0000000;

    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    cusd_client.approve(&fixture.user, &fixture.controller.address, &withdraw_amount, &100);
    
    let result = fixture.controller.withdraw_collateral(
        &protocol.id(),
        &fixture.user,
        &fixture.usdc_token,
        &withdraw_amount
    );
    
    // Verify result
    let cusd_balance = cusd_client.balance(&fixture.user);
    assert_eq!(cusd_balance, deposit_amount - withdraw_amount, "CUSD tokens not burned correctly");
    assert_eq!(result, withdraw_amount);
    
    // Check CUSD balance was reduced
}

#[test]
fn test_get_yield() {
    let fixture = TestFixture::create();
    
    let protocol1 = SupportedAdapter::BlendCapital;
    let protocol2 = SupportedAdapter::Custom(Symbol::new(&fixture.env, "OTHER"));
    
    let adapter_id1 = fixture.create_mock_lending_adapter(protocol1.clone());
    let adapter_id2 = fixture.create_mock_lending_adapter(protocol2.clone());
    
    fixture.set_adapter_mock_yield(&adapter_id1.address, &fixture.controller.address, &fixture.usdc_token, 100);
    fixture.set_adapter_mock_yield(&adapter_id2.address, &fixture.controller.address, &fixture.usdc_token, 150);
    
    let yield_amount = fixture.controller.get_yield();
    
    assert_eq!(yield_amount, 250);
}

#[test]
fn test_claim_yield() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter
    let protocol = SupportedAdapter::BlendCapital;
    let adapter_client = fixture.create_mock_lending_adapter(protocol.clone());
    
    // Set a mock yield value
    fixture.set_adapter_mock_yield(&adapter_client.address, &fixture.controller.address, &fixture.usdc_token, 100);

    // Add yield distributor members
    fixture.env.mock_all_auths_allowing_non_root_auth();
    fixture.yield_distributor.add_member(&fixture.admin, &fixture.user);
    
    // Jump forward in time to enable distribution
    fixture.jump(86400 + 10); // Distribution period + buffer

    // Mint tokens to the adapter that will be distributed
    fixture.env.mock_all_auths_allowing_non_root_auth();
    fixture.usdc_client.transfer(&fixture.user, &adapter_client.address, &100);

    // Claim yield
    // transfer usdc to the yield distributor
    fixture.env.mock_all_auths();
    let claimed_yield = fixture.controller.claim_yield();
    
    // Verify claimed amount
    assert_eq!(claimed_yield, 100);
}

#[test]
#[should_panic(expected = "Distribution not ready yet")]
fn test_claim_yield_distribution_not_available() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter
    let protocol = SupportedAdapter::BlendCapital;
    let adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
    // Set a mock yield value
    fixture.set_adapter_mock_yield(&adapter_id.address, &fixture.controller.address, &fixture.usdc_token, 100);
    
    // Claim yield without advancing time (distribution not available)
    fixture.env.mock_all_auths();
    let claimed_yield = fixture.controller.claim_yield();
    
    // Should return 0 if distribution not available
    assert_eq!(claimed_yield, 0);
}

#[test]
fn test_claim_yield_no_yield_available() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter that returns 0 yield
    let protocol = SupportedAdapter::BlendCapital;
    let adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
    // Set mock yield to 0
    fixture.set_adapter_mock_yield(&adapter_id.address, &fixture.controller.address, &fixture.usdc_token, 0);
    
    // Jump forward in time to enable distribution
    fixture.jump(86400 + 10);
    
    // Claim yield
    fixture.env.mock_all_auths();
    let claimed_yield = fixture.controller.claim_yield();
    
    // Should return 0 if no yield available
    assert_eq!(claimed_yield, 0);
}

#[test]
#[should_panic(expected = "Asset is not supported by the adapter registry")]
fn test_deposit_unsupported_asset() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter
    let protocol = SupportedAdapter::BlendCapital;
    let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
    // Create an unsupported asset
    let unsupported_token = fixture.env.register_stellar_asset_contract_v2(fixture.admin.clone());
    let unsupported_token_id = unsupported_token.address();
    
    // Try to deposit unsupported asset
    fixture.env.mock_all_auths();
    fixture.controller.deposit_collateral(
        &protocol.id(),
        &fixture.user,
        &unsupported_token_id,
        &1000_0000000
    );
}

#[test]
#[should_panic(expected = "Asset is not supported by the adapter registry")]
fn test_withdraw_unsupported_asset() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter
    let protocol = SupportedAdapter::BlendCapital;
    let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
    // Create an unsupported asset
    let unsupported_token = fixture.env.register_stellar_asset_contract_v2(fixture.admin.clone());
    let unsupported_token_id = unsupported_token.address();
    
    // Try to withdraw unsupported asset
    fixture.env.mock_all_auths();
    fixture.controller.withdraw_collateral(
        &protocol.id(),
        &fixture.user,
        &unsupported_token_id,
        &1000_0000000
    );
}

#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_deposit_without_user_auth() {
    let fixture = TestFixture::create();
    
    // Create a mock lending adapter
    let protocol = SupportedAdapter::BlendCapital;
    let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
    // Setup token approval
    let deposit_amount = 1000_0000000;

    // Don't mock user authorization
    fixture.controller.deposit_collateral(
        &protocol.id(),
        &Address::generate(&fixture.env),
        &fixture.usdc_token,
        &deposit_amount
    );
}

// #[test]
// fn test_multiple_adapters() {
//     let fixture = TestFixture::create();
    
//     // Create two mock lending adapters
//     let protocol1 = SupportedAdapter::BlendCapital;
//     let protocol2 = SupportedAdapter::Custom(Symbol::new(&fixture.env, "OTHER"));
    
//     let adapter_id1 = fixture.create_mock_lending_adapter(protocol1.clone());
//     let adapter_id2 = fixture.create_mock_lending_adapter(protocol2.clone());
    
//     // Deposit to both adapters
//     let deposit_amount1 = 1000_0000000;
//     let deposit_amount2 = 500_0000000;
    
//     fixture.approve_tokens(&fixture.user, deposit_amount1 + deposit_amount2);
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol1,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit_amount1
//     );
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol2,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit_amount2
//     );
    
//     // Verify CUSD issuance for both deposits
//     let cusd_id = fixture.cusd_manager.get_cusd_id();
//     let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
//     let cusd_balance = cusd_client.balance(&fixture.user);
//     assert_eq!(cusd_balance, deposit_amount1 + deposit_amount2);
    
//     // Set mock yields for both adapters
//     fixture.set_adapter_mock_yield(&adapter_id1, &fixture.controller.address, &fixture.usdc_token, 100);
//     fixture.set_adapter_mock_yield(&adapter_id2, &fixture.controller.address, &fixture.usdc_token, 50);
    
//     // Setup for distribution
//     fixture.env.mock_all_auths();
//     StellarAssetClient::new(&fixture.env, &fixture.usdc_token).mint(&adapter_id1, &100);
//     StellarAssetClient::new(&fixture.env, &fixture.usdc_token).mint(&adapter_id2, &50);
    
//     fixture.yield_distributor.add_member(&fixture.admin, &fixture.user);
    
//     // Jump forward to enable distribution
//     fixture.jump(86400 + 10);
    
//     // Setup token approvals
//     fixture.env.mock_all_auths();
//     fixture.usdc_client.approve(&adapter_id1, &fixture.yield_distributor.address, &100, &100000);
//     fixture.usdc_client.approve(&adapter_id2, &fixture.yield_distributor.address, &50, &100000);
    
//     // Claim yield
//     fixture.env.mock_all_auths();
//     let claimed_yield = fixture.controller.claim_yield();
    
//     // Verify claimed amount (should be sum of both adapters)
//     assert_eq!(claimed_yield, 150);
// }

// #[test]
// fn test_withdraw_all_collateral() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // First deposit collateral
//     let deposit_amount = 1000_0000000;
//     fixture.approve_tokens(&fixture.user, deposit_amount);
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit_amount
//     );
    
//     // Now withdraw all collateral
//     fixture.env.mock_all_auths();
//     let result = fixture.controller.withdraw_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit_amount
//     );
    
//     // Verify result
//     assert_eq!(result, deposit_amount);
    
//     // Check CUSD balance is zero
//     let cusd_id = fixture.cusd_manager.get_cusd_id();
//     let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
//     let cusd_balance = cusd_client.balance(&fixture.user);
//     assert_eq!(cusd_balance, 0, "All CUSD tokens should be burned");
    
//     // Check USDC was returned to user
//     let usdc_balance = fixture.usdc_client.balance(&fixture.user);
//     assert_eq!(usdc_balance, 10000_0000000, "USDC should be returned to user");
// }

// #[test]
// fn test_multi_asset_yield() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // Create a second token (like USDT)
//     let usdt_token = fixture.env.register_stellar_asset_contract_v2(fixture.admin.clone());
//     let usdt_token_id = usdt_token.address();
    
//     // Add support for USDT in the adapter
//     fixture.env.mock_all_auths();
//     fixture.adapter_registry.add_support_for_asset(
//         &fixture.admin,
//         &SupportedYieldType::Lending,
//         &protocol,
//         &usdt_token_id
//     );
    
//     // Set mock yields for different assets
//     fixture.set_adapter_mock_yield(&adapter_id, &fixture.controller.address, &fixture.usdc_token, 100);
//     fixture.set_adapter_mock_yield(&adapter_id, &fixture.controller.address, &usdt_token_id, 50);
    
//     // Setup for distribution
//     fixture.env.mock_all_auths();
//     StellarAssetClient::new(&fixture.env, &fixture.usdc_token).mint(&adapter_id, &100);
//     StellarAssetClient::new(&fixture.env, &usdt_token_id).mint(&adapter_id, &50);
    
//     // Add yield distributor member
//     fixture.env.mock_all_auths();
//     fixture.yield_distributor.add_member(&fixture.admin, &fixture.user);
    
//     // Jump forward to enable distribution
//     fixture.jump(86400 + 10);
    
//     // Setup token approvals
//     fixture.env.mock_all_auths();
//     TokenClient::new(&fixture.env, &fixture.usdc_token).approve(&adapter_id, &fixture.yield_distributor.address, &100, &100000);
//     TokenClient::new(&fixture.env, &usdt_token_id).approve(&adapter_id, &fixture.yield_distributor.address, &50, &100000);
    
//     // Claim yield
//     fixture.env.mock_all_auths();
//     let claimed_yield = fixture.controller.claim_yield();
    
//     // Verify total yield from both assets
//     assert_eq!(claimed_yield, 150);
    
//     // Verify distribution events
//     let events = fixture.env.events().all();
//     let distribution_events = events.iter().filter(|(addr, topics, _)| {
//         addr == &fixture.yield_distributor.address && 
//         topics.to_string().contains("distribute_yield")
//     }).count();
    
//     // Should be 2 distribution events (one for each asset)
//     assert_eq!(distribution_events, 2);
// }

// #[test]
// fn test_yield_calculation_during_market_changes() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // Set initial yield
//     fixture.set_adapter_mock_yield(&adapter_id, &fixture.controller.address, &fixture.usdc_token, 100);
    
//     // Check initial yield
//     let initial_yield = fixture.controller.get_yield();
//     assert_eq!(initial_yield, 100);
    
//     // Simulate market changes by modifying the yield value
//     fixture.set_adapter_mock_yield(&adapter_id, &fixture.controller.address, &fixture.usdc_token, 200);
    
//     // Check new yield value
//     let new_yield = fixture.controller.get_yield();
//     assert_eq!(new_yield, 200, "Yield should reflect market changes");
    
//     // Simulate market downturn
//     fixture.set_adapter_mock_yield(&adapter_id, &fixture.controller.address, &fixture.usdc_token, 50);
    
//     // Check yield after downturn
//     let downturn_yield = fixture.controller.get_yield();
//     assert_eq!(downturn_yield, 50, "Yield should reflect market downturn");
// }

// #[test]
// fn test_concurrent_deposits_and_withdrawals() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // Create two users
//     let user1 = fixture.user.clone();
//     let user2 = Address::generate(&fixture.env);
    
//     // Mint USDC to user2
//     fixture.env.mock_all_auths();
//     StellarAssetClient::new(&fixture.env, &fixture.usdc_token).mint(&user2, &10000_0000000);
    
//     // User1 deposits
//     let deposit_amount1 = 1000_0000000;
//     fixture.approve_tokens(&user1, deposit_amount1);
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &user1,
//         &fixture.usdc_token,
//         &deposit_amount1
//     );
    
//     // User2 deposits
//     let deposit_amount2 = 2000_0000000;
//     fixture.env.mock_all_auths();
//     fixture.usdc_client.approve(&user2, &fixture.controller.address, &deposit_amount2, &100000);
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &user2,
//         &fixture.usdc_token,
//         &deposit_amount2
//     );
    
//     // User1 withdraws half
//     let withdraw_amount1 = 500_0000000;
//     fixture.env.mock_all_auths();
//     fixture.controller.withdraw_collateral(
//         &protocol,
//         &user1,
//         &fixture.usdc_token,
//         &withdraw_amount1
//     );
    
//     // User2 withdraws a quarter
//     let withdraw_amount2 = 500_0000000;
//     fixture.env.mock_all_auths();
//     fixture.controller.withdraw_collateral(
//         &protocol,
//         &user2,
//         &fixture.usdc_token,
//         &withdraw_amount2
//     );
    
//     // Verify CUSD balances
//     let cusd_id = fixture.cusd_manager.get_cusd_id();
//     let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
    
//     let cusd_balance1 = cusd_client.balance(&user1);
//     let cusd_balance2 = cusd_client.balance(&user2);
    
//     assert_eq!(cusd_balance1, deposit_amount1 - withdraw_amount1, "User1 CUSD balance incorrect");
//     assert_eq!(cusd_balance2, deposit_amount2 - withdraw_amount2, "User2 CUSD balance incorrect");
// }

// #[test]
// fn test_zero_deposit_and_withdraw() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // Try to deposit zero amount
//     fixture.env.mock_all_auths();
//     let result = fixture.controller.deposit_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &0
//     );
    
//     // Should succeed with zero result
//     assert_eq!(result, 0);
    
//     // Try to withdraw zero amount
//     fixture.env.mock_all_auths();
//     let result = fixture.controller.withdraw_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &0
//     );
    
//     // Should succeed with zero result
//     assert_eq!(result, 0);
// }

// #[test]
// fn test_multiple_deposits_single_asset() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let _adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // Make multiple deposits
//     let deposit1 = 100_0000000;
//     let deposit2 = 200_0000000;
//     let deposit3 = 300_0000000;
    
//     // Approve tokens
//     fixture.approve_tokens(&fixture.user, deposit1 + deposit2 + deposit3);
    
//     // Make deposits
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit1
//     );
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit2
//     );
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit3
//     );
    
//     // Verify CUSD balance
//     let cusd_id = fixture.cusd_manager.get_cusd_id();
//     let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
//     let cusd_balance = cusd_client.balance(&fixture.user);
    
//     assert_eq!(cusd_balance, deposit1 + deposit2 + deposit3, "CUSD balance should be sum of all deposits");
// }

// #[test]
// fn test_full_lifecycle_with_yield() {
//     let fixture = TestFixture::create();
    
//     // Create a mock lending adapter
//     let protocol = SupportedAdapter::BlendCapital;
//     let adapter_id = fixture.create_mock_lending_adapter(protocol.clone());
    
//     // Make deposit
//     let deposit_amount = 1000_0000000;
//     fixture.approve_tokens(&fixture.user, deposit_amount);
    
//     fixture.env.mock_all_auths();
//     fixture.controller.deposit_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit_amount
//     );
    
//     // Set yield
//     let yield_amount = 100_0000000;
//     fixture.set_adapter_mock_yield(&adapter_id, &fixture.controller.address, &fixture.usdc_token, yield_amount);
    
//     // Setup for distribution
//     fixture.env.mock_all_auths();
//     StellarAssetClient::new(&fixture.env, &fixture.usdc_token).mint(&adapter_id, &yield_amount);
//     fixture.yield_distributor.add_member(&fixture.admin, &fixture.user);
    
//     // Jump forward in time
//     fixture.jump(86400 + 10);
    
//     // Setup token approval
//     fixture.env.mock_all_auths();
//     fixture.usdc_client.approve(&adapter_id, &fixture.yield_distributor.address, &yield_amount, &100000);
    
//     // Claim yield
//     fixture.env.mock_all_auths();
//     let claimed = fixture.controller.claim_yield();
//     assert_eq!(claimed, yield_amount);
    
//     // Withdraw original deposit
//     fixture.env.mock_all_auths();
//     let withdrawn = fixture.controller.withdraw_collateral(
//         &protocol,
//         &fixture.user,
//         &fixture.usdc_token,
//         &deposit_amount
//     );
//     assert_eq!(withdrawn, deposit_amount);
    
//     // Verify CUSD balance is zero
//     let cusd_id = fixture.cusd_manager.get_cusd_id();
//     let cusd_client = TokenClient::new(&fixture.env, &cusd_id);
//     let cusd_balance = cusd_client.balance(&fixture.user);
//     assert_eq!(cusd_balance, 0, "All CUSD tokens should be burned");
// }
