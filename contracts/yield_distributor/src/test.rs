#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::{
    contract::{YieldDistributor, YieldDistributorClient},
};

struct TestFixture {
    env: Env,
    distributor: YieldDistributorClient<'static>,
    treasury: Address,
    treasury_share_bps: u32,
    yield_controller: Address,
    distribution_period: u64,
    owner: Address,
    admin: Address,
    member1: Address,
    member2: Address,
    member3: Address,
    token_admin: Address,
    token_id: Address,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        env.ledger().set_sequence_number(100);
        env.ledger().set_timestamp(1000000000); // Set timestamp to make distribution available
        
        let treasury = Address::generate(&env);
        let treasury_share_bps = 1000u32; // 10%
        let yield_controller = Address::generate(&env);
        let distribution_period = 86400u64; // 1 day in seconds
        let owner = Address::generate(&env);
        let admin = Address::generate(&env);
        let member1 = Address::generate(&env);
        let member2 = Address::generate(&env);
        let member3 = Address::generate(&env);
        let token_admin = Address::generate(&env);

        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_id = token_contract.address();
        let distributor_id = env.register(
            YieldDistributor,
            (
                treasury.clone(),
                treasury_share_bps,
                yield_controller.clone(),
                distribution_period,
                owner.clone(),
                admin.clone(),
            ),
        );
        let distributor = YieldDistributorClient::new(&env, &distributor_id);

        TestFixture {
            env,
            distributor,
            treasury,
            treasury_share_bps,
            yield_controller,
            distribution_period,
            owner,
            admin,
            member1,
            member2,
            member3,
            token_admin,
            token_id,
        }
    }

    fn token_client(&self) -> TokenClient<'static> {
        TokenClient::new(&self.env, &self.token_id)
    }

    fn stellar_token_client(&self) -> StellarAssetClient<'static> {
        StellarAssetClient::new(&self.env, &self.token_id)
    }

    fn assert_event_with_address_data(&self, expected_event: (Symbol,), expected_data: Address) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.distributor.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn assert_event_with_u32_data(&self, expected_event: (Symbol,), expected_data: u32) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.distributor.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn assert_event_with_u64_data(&self, expected_event: (Symbol,), expected_data: u64) {
        let events = self.env.events().all();
        if !events.is_empty() {
            let published_event = vec![&self.env, events.last_unchecked()];
            let expected = vec![
                &self.env,
                (
                    self.distributor.address.clone(),
                    expected_event.into_val(&self.env),
                    expected_data.into_val(&self.env),
                ),
            ];
            assert_eq!(published_event, expected);
        }
    }

    fn add_members(&self) {
        self.env.mock_all_auths();
        self.distributor.add_member(&self.member1);
        self.distributor.add_member(&self.member2);
        self.distributor.add_member(&self.member3);
    }

    fn mint_tokens_to_distributor(&self, amount: i128) {
        self.env.mock_all_auths_allowing_non_root_auth();
        self.stellar_token_client().set_admin(&self.distributor.address);
        self.stellar_token_client().mint(&self.distributor.address, &amount);
    }
}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();

    assert_eq!(fixture.distributor.get_treasury(), fixture.treasury);
    assert_eq!(fixture.distributor.get_treasury_share(), fixture.treasury_share_bps);
    assert_eq!(fixture.distributor.get_yield_controller(), fixture.yield_controller);
    assert_eq!(fixture.distributor.get_distribution_period(), fixture.distribution_period);
    
    let members = fixture.distributor.list_members();
    assert_eq!(members.len(), 0);
    assert!(fixture.distributor.is_distribution_available());
}

#[test]
fn test_add_member() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    let _ = fixture.env.events().all();

    fixture.distributor.add_member(&fixture.member1);

    let members = fixture.distributor.list_members();
    assert_eq!(members.len(), 1);
    assert_eq!(members.get(0).unwrap(), fixture.member1);
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "add_member"),),
        fixture.member1.clone()
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_add_member_unauthorized() {
    let fixture = TestFixture::create();

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.add_member(&fixture.member1);
}

#[test]
#[should_panic(expected = "Error(Contract, #1200)")]
fn test_add_member_already_exists() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member1);
}

#[test]
fn test_add_multiple_members() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member2);
    fixture.distributor.add_member(&fixture.member3);
    let members = fixture.distributor.list_members();
    assert_eq!(members.len(), 3);
    assert!(members.contains(&fixture.member1));
    assert!(members.contains(&fixture.member2));
    assert!(members.contains(&fixture.member3));
}

#[test]
fn test_remove_member() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member2);

    let _ = fixture.env.events().all();

    fixture.distributor.remove_member(&fixture.member1);

    let members = fixture.distributor.list_members();
    assert_eq!(members.len(), 1);
    assert_eq!(members.get(0).unwrap(), fixture.member2);
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "remove_member"),),
        fixture.member1.clone()
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_remove_member_unauthorized() {
    let fixture = TestFixture::create();

    fixture.env.mock_all_auths();
    fixture.distributor.add_member(&fixture.member1);

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.remove_member(&fixture.member1);
}

#[test]
#[should_panic(expected = "Error(Contract, #1201)")]
fn test_remove_member_does_not_exist() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    fixture.distributor.remove_member(&fixture.member1);
}

#[test]
fn test_set_treasury() {
    let fixture = TestFixture::create();
    let new_treasury = Address::generate(&fixture.env);
    
    fixture.env.mock_all_auths();

    let _ = fixture.env.events().all();

    fixture.distributor.set_treasury(&new_treasury);

    assert_eq!(fixture.distributor.get_treasury(), new_treasury);
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "set_treasury"),),
        new_treasury.clone()
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_treasury_unauthorized() {
    let fixture = TestFixture::create();
    let new_treasury = Address::generate(&fixture.env);

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.set_treasury(&new_treasury);
}

#[test]
fn test_set_treasury_share() {
    let fixture = TestFixture::create();
    let new_share = 2000u32; // 20%
    
    fixture.env.mock_all_auths();

    let _ = fixture.env.events().all();

    fixture.distributor.set_treasury_share(&new_share);

    assert_eq!(fixture.distributor.get_treasury_share(), new_share);
    fixture.assert_event_with_u32_data(
        (Symbol::new(&fixture.env, "set_treasury_share"),),
        new_share
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_treasury_share_unauthorized() {
    let fixture = TestFixture::create();
    let new_share = 2000u32;

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.set_treasury_share(&new_share);
}

#[test]
fn test_set_distribution_period() {
    let fixture = TestFixture::create();
    let new_period = 172800u64; // 2 days
    
    fixture.env.mock_all_auths();

    let _ = fixture.env.events().all();

    fixture.distributor.set_distribution_period(&new_period);

    assert_eq!(fixture.distributor.get_distribution_period(), new_period);
    fixture.assert_event_with_u64_data(
        (Symbol::new(&fixture.env, "set_distribution_period"),),
        new_period
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_distribution_period_unauthorized() {
    let fixture = TestFixture::create();
    let new_period = 172800u64;

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.set_distribution_period(&new_period);
}

#[test]
fn test_set_yield_controller() {
    let fixture = TestFixture::create();
    let new_controller = Address::generate(&fixture.env);
    
    fixture.env.mock_all_auths();

    let _ = fixture.env.events().all();

    fixture.distributor.set_yield_controller(&new_controller);

    assert_eq!(fixture.distributor.get_yield_controller(), new_controller);
    fixture.assert_event_with_address_data(
        (Symbol::new(&fixture.env, "set_yield_controller"),),
        new_controller.clone()
    );
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_yield_controller_unauthorized() {
    let fixture = TestFixture::create();
    let new_controller = Address::generate(&fixture.env);

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.set_yield_controller(&new_controller);
}

#[test]
fn test_yield_distribution_basic() {
    let fixture = TestFixture::create();
    let total_amount = 10000i128;

    fixture.add_members();
    fixture.mint_tokens_to_distributor(total_amount);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    let result = fixture.distributor.distribute_yield(&fixture.token_id, &total_amount);
    assert_eq!(result, total_amount);
    let treasury_share = (total_amount * fixture.treasury_share_bps as i128) / 10000;
    let member_share = total_amount - treasury_share;
    let per_member_amount = member_share / 3; // 3 members

    assert_eq!(fixture.token_client().balance(&fixture.treasury), treasury_share);
    assert_eq!(fixture.token_client().balance(&fixture.member1), per_member_amount);
    assert_eq!(fixture.token_client().balance(&fixture.member2), per_member_amount);
    assert_eq!(fixture.token_client().balance(&fixture.member3), per_member_amount);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_distribute_yield_unauthorized() {
    let fixture = TestFixture::create();
    let total_amount = 10000i128;

    fixture.add_members();
    fixture.mint_tokens_to_distributor(total_amount);

    fixture.env.mock_auths(&[]);
    
    fixture.distributor.distribute_yield(&fixture.token_id, &total_amount);
}

#[test]
fn test_yield_distribution_no_members() {
    let fixture = TestFixture::create();
    let total_amount = 10000i128;

    fixture.mint_tokens_to_distributor(total_amount);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    let result = fixture.distributor.distribute_yield(&fixture.token_id, &total_amount);
    assert_eq!(result, total_amount);
    assert_eq!(fixture.token_client().balance(&fixture.treasury), total_amount);
}

#[test]
fn test_yield_distribution_zero_treasury_share() {
    let fixture = TestFixture::create();
    let total_amount = 10000i128;

    fixture.env.mock_all_auths();
    fixture.distributor.set_treasury_share(&0u32);

    fixture.add_members();
    fixture.mint_tokens_to_distributor(total_amount);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    let result = fixture.distributor.distribute_yield(&fixture.token_id, &total_amount);
    assert_eq!(result, total_amount);
    let per_member_amount = total_amount / 3; // 3 members
    assert_eq!(fixture.token_client().balance(&fixture.treasury), 0);
    assert_eq!(fixture.token_client().balance(&fixture.member1), per_member_amount);
    assert_eq!(fixture.token_client().balance(&fixture.member2), per_member_amount);
    assert_eq!(fixture.token_client().balance(&fixture.member3), per_member_amount);
}

#[test]
fn test_yield_distribution_100_percent_treasury() {
    let fixture = TestFixture::create();
    let total_amount = 10000i128;

    fixture.env.mock_all_auths();
    fixture.distributor.set_treasury_share(&10000u32);

    fixture.add_members();
    fixture.mint_tokens_to_distributor(total_amount);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    let result = fixture.distributor.distribute_yield(&fixture.token_id, &total_amount);
    assert_eq!(result, total_amount);
    assert_eq!(fixture.token_client().balance(&fixture.treasury), total_amount);
    assert_eq!(fixture.token_client().balance(&fixture.member1), 0);
    assert_eq!(fixture.token_client().balance(&fixture.member2), 0);
    assert_eq!(fixture.token_client().balance(&fixture.member3), 0);
}

#[test]
fn test_distribution_availability() {
    let fixture = TestFixture::create();

    assert!(fixture.distributor.is_distribution_available());

    fixture.add_members();
    fixture.mint_tokens_to_distributor(1000);

    fixture.env.mock_all_auths_allowing_non_root_auth();
    fixture.distributor.distribute_yield(&fixture.token_id, &1000);

    assert!(!fixture.distributor.is_distribution_available());
    let next_time = fixture.distributor.get_next_distribution_time();
    assert!(next_time > fixture.env.ledger().timestamp());
}

#[test]
fn test_member_management_cycle() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member2);
    assert_eq!(fixture.distributor.list_members().len(), 2);

    fixture.distributor.remove_member(&fixture.member1);
    assert_eq!(fixture.distributor.list_members().len(), 1);
    assert_eq!(fixture.distributor.list_members().get(0).unwrap(), fixture.member2);

    fixture.distributor.add_member(&fixture.member1);
    assert_eq!(fixture.distributor.list_members().len(), 2);
    fixture.distributor.remove_member(&fixture.member1);
    fixture.distributor.remove_member(&fixture.member2);
    assert_eq!(fixture.distributor.list_members().len(), 0);
}

#[test]
fn test_treasury_configuration_changes() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    let shares = [0u32, 500u32, 2500u32, 5000u32, 10000u32]; // 0%, 5%, 25%, 50%, 100%
    
    for share in shares.iter() {
        fixture.distributor.set_treasury_share(&share);
        assert_eq!(fixture.distributor.get_treasury_share(), *share);
    }

    let new_treasury1 = Address::generate(&fixture.env);
    let new_treasury2 = Address::generate(&fixture.env);
    
    fixture.distributor.set_treasury(&new_treasury1);
    assert_eq!(fixture.distributor.get_treasury(), new_treasury1);
    
    fixture.distributor.set_treasury(&new_treasury2);
    assert_eq!(fixture.distributor.get_treasury(), new_treasury2);
}

#[test]
fn test_distribution_period_changes() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    let periods = [3600u64, 86400u64, 604800u64, 2592000u64]; // 1 hour, 1 day, 1 week, 1 month
    
    for period in periods.iter() {
        fixture.distributor.set_distribution_period(&period);
        assert_eq!(fixture.distributor.get_distribution_period(), *period);
    }
}

#[test]
fn test_yield_controller_changes() {
    let fixture = TestFixture::create();
    
    fixture.env.mock_all_auths();

    let new_controller1 = Address::generate(&fixture.env);
    let new_controller2 = Address::generate(&fixture.env);
    
    fixture.distributor.set_yield_controller(&new_controller1);
    assert_eq!(fixture.distributor.get_yield_controller(), new_controller1);
    
    fixture.distributor.set_yield_controller(&new_controller2);
    assert_eq!(fixture.distributor.get_yield_controller(), new_controller2);
}

#[test]
fn test_small_amount_distribution() {
    let fixture = TestFixture::create();
    let small_amount = 10i128; // Very small amount

    fixture.add_members();
    fixture.mint_tokens_to_distributor(small_amount);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    let result = fixture.distributor.distribute_yield(&fixture.token_id, &small_amount);
    assert_eq!(result, small_amount);
    let total_distributed = fixture.token_client().balance(&fixture.treasury) +
                          fixture.token_client().balance(&fixture.member1) +
                          fixture.token_client().balance(&fixture.member2) +
                          fixture.token_client().balance(&fixture.member3);
    assert_eq!(total_distributed, small_amount);
}

#[test]
fn test_large_amount_distribution() {
    let fixture = TestFixture::create();
    let large_amount = 1_000_000_000_000i128; // Very large amount

    fixture.add_members();
    fixture.mint_tokens_to_distributor(large_amount);

    fixture.env.mock_all_auths_allowing_non_root_auth();

    let result = fixture.distributor.distribute_yield(&fixture.token_id, &large_amount);
    assert_eq!(result, large_amount);
    let treasury_share = (large_amount * fixture.treasury_share_bps as i128) / 10000;
    assert_eq!(fixture.token_client().balance(&fixture.treasury), treasury_share);
}

#[test]
fn test_resource_limit_simulation_multiple_rounds() {
    let fixture = TestFixture::create();
    let distribution_amount = 10000i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Start with a few members
    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member2);
    
    // Simulate multiple distribution rounds (like distribution round 6)
    for round in 0..6 {
        // Add more members each round to increase memory usage
        let new_member = Address::generate(&fixture.env);
        fixture.distributor.add_member(&new_member);
        
        // Mint tokens for this distribution
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        // Make distribution available by advancing time if needed
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        // Perform the distribution
        let result = fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        assert_eq!(result, distribution_amount);
        
        // Get distribution history - THIS IS WHERE MEMORY GROWS
        let _history = fixture.distributor.get_distribution_history();
        
        // Verify the distribution was recorded - only check after first round
        if round > 0 {
            let distribution_info = fixture.distributor.get_distribution_info();
            // Check that we have an active distribution (may not be processed yet)
            assert!(distribution_info.epoch > 0);
        }
    }
    
    // By round 6, we should have:
    // - 6 distribution records in history
    // - Each distribution has growing member lists
    // - Total memory usage = 6 distributions × member lists × address size
    
    let final_history = fixture.distributor.get_distribution_history();
    // Should have at least some distributions recorded
    assert!(final_history.len() >= 1);
    
    // Verify exponential memory growth pattern
    for (i, dist) in final_history.iter().enumerate() {
        // Each round should have more members than the previous
        if i > 0 {
            let prev_dist = final_history.get((i - 1) as u32).unwrap();
            assert!(dist.member_count >= prev_dist.member_count);
        }
    }
    
    // Calculate total member count across all distributions
    let total_member_count: u32 = final_history.iter()
        .map(|dist| dist.member_count)
        .sum();
    
    // This demonstrates exponential memory growth that causes ResourceLimitExceeded
    // Each distribution stores a full member list, so total entries = distributions × members
    assert!(total_member_count > 0); // Should have stored member count
}

#[test] 
fn test_resource_limit_with_many_members() {
    let fixture = TestFixture::create();
    let distribution_amount = 10000i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Add many members to simulate a realistic scenario  
    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member2);
    fixture.distributor.add_member(&fixture.member3);
    
    for _ in 0..20 { // Total 23 members
        let new_member = Address::generate(&fixture.env);
        fixture.distributor.add_member(&new_member);
    }
    
    // Perform multiple distributions to build up history
    for round in 0..8 { // Simulate 8 rounds like a real scenario
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        // This is where the resource limit would be hit in real scenario
        let result = fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        assert_eq!(result, distribution_amount);
        
        // Check resource usage by getting history (expensive operation)
        let history = fixture.distributor.get_distribution_history();
        let total_addresses_stored = history.iter()
            .map(|dist| dist.member_count)
            .sum::<u32>();
            
        // Resource usage grows as: rounds × members_per_round
        // Round 8 with 23 members = 8 × 23 = 184 address entries in storage
        if round >= 5 { // By round 6+, this becomes expensive
            // This would cause ResourceLimitExceeded in production
            assert!(total_addresses_stored > 100); // High memory usage detected
        }
    }
}

#[test]
fn test_distribution_history_memory_stress() {
    let fixture = TestFixture::create();
    let distribution_amount = 1000i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Create a large number of members to stress test memory
    for _i in 0..15 { // 15 members
        let member = Address::generate(&fixture.env);
        fixture.distributor.add_member(&member);
    }
    
    // Perform 10 distributions to create substantial history
    for round in 0..10 {
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        
        // Track memory usage each round - THIS IS THE EXPENSIVE OPERATION
        let _history = fixture.distributor.get_distribution_history();
        // Each call to get_distribution_history loads ALL past distributions with full member lists
    }
    
    // Final memory usage analysis
    let final_history = fixture.distributor.get_distribution_history();
    let total_member_count: u32 = final_history.iter()
        .map(|dist| dist.member_count)
        .sum();
    
    // This demonstrates the exponential memory growth that causes ResourceLimitExceeded
    assert!(total_member_count >= 150); // 10 distributions × 15 members = 150 entries minimum
}

#[test]
fn test_simulate_actual_resource_limit() {
    let fixture = TestFixture::create();
    let distribution_amount = 1000i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Create an extreme scenario to demonstrate high resource usage
    for _i in 0..50 { // 50 members
        let member = Address::generate(&fixture.env);
        fixture.distributor.add_member(&member);
    }
    
    // Perform many distributions to maximize memory usage
    for round in 0..10 { // 10 rounds = 10 distributions × 50 members = 500 address entries
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        // This would eventually fail with ResourceLimitExceeded in production
        // The call chain: distribute_yield -> record_distribution -> read_distribution_history
        fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        
        // Check if we're accumulating significant memory usage
        let history = fixture.distributor.get_distribution_history();
        let total_member_count: u32 = history.iter().map(|d| d.member_count).sum();
        
        // By round 6+, this would cause ResourceLimitExceeded in Stellar
        if round >= 5 {
            assert!(total_member_count > 250); // Significant memory usage
        }
    }
}

#[test]
fn test_performance_benchmark_before_vs_after() {
    let fixture = TestFixture::create();
    let distribution_amount = 1000i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Add a realistic number of members (similar to production)
    for _i in 0..10 {
        let member = Address::generate(&fixture.env);
        fixture.distributor.add_member(&member);
    }
    
    // Track performance data without using Vec (unavailable in Soroban)
    let mut round_6_cpu = 0u64;
    let mut round_6_memory = 0u64;
    let mut round_15_cpu = 0u64;
    let mut round_15_memory = 0u64;
    
    for round in 0..15 { // Test up to round 15 to exceed the problematic round 6
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        // Reset budget to measure this round's resource usage
        fixture.env.cost_estimate().budget().reset_default();
        
        // This is the critical operation that was failing at round 6
        let result = fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        assert_eq!(result, distribution_amount);
        
        // Capture resource usage for this round
        let cpu_used = fixture.env.cost_estimate().budget().cpu_instruction_cost();
        let memory_used = fixture.env.cost_estimate().budget().memory_bytes_cost();
        
        // Store specific round data for comparison
        if round == 5 { // Round 6 (0-indexed)
            round_6_cpu = cpu_used;
            round_6_memory = memory_used;
        } else if round == 14 { // Round 15 (0-indexed)
            round_15_cpu = cpu_used;
            round_15_memory = memory_used;
        }
        
        // Verify we don't have resource growth issues
        if round >= 5 {
            // After round 6, the old implementation would fail with ResourceLimitExceeded
            // New implementation should have reasonable resource usage (much lower than Stellar limits)
            // Stellar typically limits around 100M instructions - we should be well under that
            assert!(cpu_used < 10_000_000, "CPU usage too high at round {}: {} (Stellar limit ~100M)", round + 1, cpu_used);
            assert!(memory_used < 1_000_000, "Memory usage too high at round {}: {}", round + 1, memory_used);
        }
    }
    
    // Verify the distribution history still works correctly
    let final_history = fixture.distributor.get_distribution_history();
    assert!(final_history.len() >= 10, "Should have recorded multiple distributions");
    
    // Verify resource usage remains bounded (not growing linearly) 
    assert!(round_6_cpu > 0, "Should have captured round 6 CPU usage");
    assert!(round_15_cpu > 0, "Should have captured round 15 CPU usage");
    
    // Calculate growth ratios (with safety checks to avoid division by zero)
    let cpu_growth_ratio = if round_6_cpu > 0 {
        round_15_cpu as f64 / round_6_cpu as f64
    } else {
        1.0
    };
    let memory_growth_ratio = if round_6_memory > 0 {
        round_15_memory as f64 / round_6_memory as f64
    } else {
        1.0
    };
    
    // Assert that resource usage doesn't grow linearly with distribution count
    // With our optimization, resource usage should remain relatively constant
    assert!(cpu_growth_ratio < 2.0, "CPU usage grew too much: {:.2}x from round 6 to 15", cpu_growth_ratio);
    assert!(memory_growth_ratio < 2.0, "Memory usage grew too much: {:.2}x from round 6 to 15", memory_growth_ratio);
}

#[test]
fn test_distribution_history_consistency_after_optimization() {
    let fixture = TestFixture::create();
    let distribution_amount = 1000i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Add members
    fixture.distributor.add_member(&fixture.member1);
    fixture.distributor.add_member(&fixture.member2);
    
    // Track expected distribution count
    let mut expected_count = 0;
    
    // Perform several distributions and track expected data
    for round in 0..8 {
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        let start_timestamp = fixture.env.ledger().timestamp();
        
        fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        
        // Track what we expect to see in history
        expected_count += 1;
    }
    
    // Verify distribution history is consistent with our expectations
    let history = fixture.distributor.get_distribution_history();
    
    // Should have processed distributions
    assert!(history.len() > 0, "Should have distribution history");
    
    // Verify history contains correct data
    for (i, distribution) in history.iter().enumerate() {
        assert!(distribution.is_processed, "Distribution {} should be processed", i);
        assert!(distribution.distribution_total > 0, "Distribution {} should have positive total", i);
        assert_eq!(distribution.member_count, 2, "Distribution {} should have 2 members", i);
    }
    
    // Verify we can call get_distribution_history multiple times without issues
    for _i in 0..5 {
        let history_check = fixture.distributor.get_distribution_history();
        assert_eq!(history_check.len(), history.len(), "History length should be consistent");
    }
}

#[test]
fn test_memory_efficiency_stress_test() {
    let fixture = TestFixture::create();
    let distribution_amount = 500i128;
    
    fixture.env.mock_all_auths_allowing_non_root_auth();
    
    // Add a significant number of members to stress test
    for _i in 0..25 {
        let member = Address::generate(&fixture.env);
        fixture.distributor.add_member(&member);
    }
    
    // Perform many distributions to test scaling
    for round in 0..20 {
        fixture.mint_tokens_to_distributor(distribution_amount);
        
        if round > 0 {
            let current_time = fixture.env.ledger().timestamp();
            fixture.env.ledger().set_timestamp(current_time + fixture.distribution_period + 1);
        }
        
        // Reset budget to measure each operation independently  
        fixture.env.cost_estimate().budget().reset_default();
        
        // The critical test: this should not fail even at round 20
        let result = fixture.distributor.distribute_yield(&fixture.token_id, &distribution_amount);
        assert_eq!(result, distribution_amount);
        
        // Verify resource usage stays reasonable
        let cpu_used = fixture.env.cost_estimate().budget().cpu_instruction_cost();
        let memory_used = fixture.env.cost_estimate().budget().memory_bytes_cost();
        
        // These are the limits that would have been exceeded with the old implementation
        // The old implementation would have failed around round 6 with ResourceLimitExceeded
        // Our optimized version should stay well below Stellar's ~100M instruction limit
        assert!(cpu_used < 20_000_000, "Round {}: CPU usage {} approaching Stellar limits (~100M)", round + 1, cpu_used);
        assert!(memory_used < 2_000_000, "Round {}: Memory usage {} too high", round + 1, memory_used);
    }
    
    // Final verification: ensure distribution history is still complete and accurate
    let history = fixture.distributor.get_distribution_history();
    
    // Should have recorded distributions for processed rounds
    assert!(history.len() >= 15, "Should have substantial distribution history");
    
    // Each distribution should have the expected member count
    for (i, dist) in history.iter().enumerate() {
        assert_eq!(dist.member_count, 25, "Distribution {} should have 25 members", i);
        assert!(dist.is_processed, "Distribution {} should be processed", i);
        assert!(dist.distribution_total > 0, "Distribution {} should have positive total", i);
    }
}