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