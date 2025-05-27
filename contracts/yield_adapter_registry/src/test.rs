#![cfg(test)]
extern crate std;

use pretty_assertions::assert_eq;
use soroban_sdk::{
    log, symbol_short,
    testutils::{Address as _, Events},
    vec, Address, Env, IntoVal, Symbol,
};

use crate::{
    contract::{YieldAdapterRegistry, YieldAdapterRegistryArgs, YieldAdapterRegistryClient},
    storage_types::YieldAdapterRegistryMap,
};

use yield_adapter::contract_types::{SupportedAdapter, SupportedYieldType};

struct TestFixture {
    env: Env,
    registry: YieldAdapterRegistryClient<'static>,
    admin: Address,
    user: Address,
}

impl TestFixture {
    fn create() -> Self {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Deploy the registry contract
        let registry_id = env.register(
            YieldAdapterRegistry,
            YieldAdapterRegistryArgs::__constructor(&admin),
        );

        let registry = YieldAdapterRegistryClient::new(&env, &registry_id);

        Self {
            env,
            registry,
            admin,
            user,
        }
    }

    // Helper to create a protocol adapter
    fn create_adapter(&self) -> (Address, SupportedAdapter) {
        let adapter_address = Address::generate(&self.env);
        (adapter_address, SupportedAdapter::BlendCapital)
    }

    // Helper to create an asset
    fn create_asset(&self) -> Address {
        Address::generate(&self.env)
    }

    // Helper to verify an adapter exists in storage
    fn verify_adapter_exists(
        &self,
        protocol: SupportedAdapter,
        expected_address: &Address,
    ) -> bool {
        self.env.as_contract(&self.registry.address, || {
            if let Some(registry_map) = self
                .env
                .storage()
                .persistent()
                .get::<Symbol, YieldAdapterRegistryMap>(&SupportedYieldType::Lending.id())
            {
                if registry_map.contains_key(protocol.id()) {
                    let stored_address = registry_map.get_adapter(protocol.id());
                    return stored_address == *expected_address;
                }
            }
            false
        })
    }

    // Helper to verify an asset is supported
    fn verify_asset_supported(&self, protocol: SupportedAdapter, asset: &Address) -> bool {
        self.env.as_contract(&self.registry.address, || {
            if let Some(registry_map) = self
                .env
                .storage()
                .persistent()
                .get::<Symbol, YieldAdapterRegistryMap>(&SupportedYieldType::Lending.id())
            {
                return registry_map.supports_asset(protocol.id(), asset.clone());
            }
            false
        })
    }
}

#[test]
fn test_constructor() {
    let fixture = TestFixture::create();

    // Verify admin role was granted to the admin address
    fixture.env.mock_all_auths();

    // The admin should be able to call admin-only functions
    let (adapter_address, protocol) = fixture.create_adapter();
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );

    // Verify adapter is registered in storage
    assert!(fixture.verify_adapter_exists(protocol, &adapter_address));
}

// Test admin role management
#[test]
fn test_set_yield_adapter_admin() {
    let fixture = TestFixture::create();
    let new_admin = Address::generate(&fixture.env);

    // Mock authorization for the admin
    fixture.env.mock_all_auths();

    // Set new admin
    fixture
        .registry
        .set_yield_adapter_admin(&fixture.admin, &new_admin);

    // Expected event has topic "set_admin" and data is the new admin address
    let published_event = vec![&fixture.env, fixture.env.events().all().last_unchecked()];
    let expected_event = vec![
        &fixture.env,
        (
            fixture.registry.address.clone(),
            (Symbol::new(&fixture.env, "set_admin"),).into_val(&fixture.env),
            new_admin.into_val(&fixture.env),
        ),
    ];
    assert_eq!(published_event, expected_event);

    // New admin should now be able to register adapters
    let (adapter_address, protocol) = fixture.create_adapter();
    fixture.registry.register_adapter(
        &new_admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );

    // Verify adapter is registered in storage
    assert!(fixture.verify_adapter_exists(protocol, &adapter_address));
}

// Test adapter registration
#[test]
fn test_register_adapter() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();

    // Mock authorization for the admin
    fixture.env.mock_all_auths();

    // Clear events before registration
    let _ = fixture.env.events().all();

    // Register adapter
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );

    // Verify event emission
    log!(&fixture.env, "All events: {:?}", fixture.env.events().all());
    let published_event = vec![&fixture.env, fixture.env.events().all().last_unchecked()];
    let expected_event = vec![
        &fixture.env,
        (
            fixture.registry.address.clone(),
            (
                Symbol::new(&fixture.env, "register_adapter"),
                SupportedYieldType::Lending.id(),
            )
                .into_val(&fixture.env),
            (protocol.id().clone(), adapter_address.clone()).into_val(&fixture.env),
        ),
    ];
    assert_eq!(published_event, expected_event);

    // Verify adapter is registered in storage
    assert!(fixture.verify_adapter_exists(protocol.clone(), &adapter_address));

    // Test getting the adapter
    let retrieved_address = fixture
        .registry
        .get_adapter(&SupportedYieldType::Lending.id(), &protocol.id());
    assert_eq!(retrieved_address, adapter_address);
}

// Test unauthorized adapter registration (should fail)
#[test]
#[should_panic(expected = "Error(Contract, #1301)")]
fn test_register_adapter_unauthorized() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();

    // This should fail with an access control error
    fixture.env.mock_all_auths();
    fixture.registry.register_adapter(
        &fixture.user,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );
}

#[test]
fn test_remove_adapter() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();

    fixture.env.mock_all_auths();

    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );
    assert!(fixture.verify_adapter_exists(protocol.clone(), &adapter_address));

    let _ = fixture.env.events().all();

    fixture.registry.remove_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
    );

    let published_event = vec![&fixture.env, fixture.env.events().all().last_unchecked()];
    let expected_event = vec![
        &fixture.env,
        (
            fixture.registry.address.clone(),
            (
                Symbol::new(&fixture.env, "remove_adapter"),
                SupportedYieldType::Lending.id(),
            )
                .into_val(&fixture.env),
            (protocol.id().clone(), adapter_address.clone()).into_val(&fixture.env),
        ),
    ];
    assert_eq!(published_event, expected_event);

    assert!(!fixture.verify_adapter_exists(protocol, &adapter_address));
}

#[test]
#[should_panic(expected = "Error(Contract, #1301)")]
fn test_remove_adapter_unauthorized() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();

    fixture.env.mock_all_auths();
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );

    fixture.registry.remove_adapter(
        &fixture.user,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1100)")]
fn test_get_non_existent_adapter() {
    let fixture = TestFixture::create();
    let protocol = SupportedAdapter::BlendCapital;

    // This should panic with "Yield adapter not found"
    fixture
        .registry
        .get_adapter(&SupportedYieldType::Lending.id(), &protocol.id());
}

#[test]
fn test_add_support_for_asset() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();
    let asset = fixture.create_asset();

    // Mock authorization for the admin
    fixture.env.mock_all_auths();

    // Register adapter first
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );

    // Clear events before adding asset support
    let _ = fixture.env.events().all();

    // Add asset support
    fixture.registry.add_support_for_asset(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );

    // Verify event emission
    let published_event = vec![&fixture.env, fixture.env.events().all().last_unchecked()];
    let expected_event = vec![
        &fixture.env,
        (
            fixture.registry.address.clone(),
            (
                Symbol::new(&fixture.env, "add_support_for_asset"),
                SupportedYieldType::Lending.id(),
            )
                .into_val(&fixture.env),
            (protocol.id().clone(), asset.clone()).into_val(&fixture.env),
        ),
    ];
    assert_eq!(published_event, expected_event);

    // Verify asset is supported
    assert!(fixture.verify_asset_supported(protocol.clone(), &asset));
    let is_supported = fixture.registry.is_supported_asset(
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );
    assert!(is_supported);
}

#[test]
#[should_panic(expected = "Error(Contract, #1301)")]
fn test_add_support_for_asset_unauthorized() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();
    let asset = fixture.create_asset();

    // Mock authorization for the admin to register adapter
    fixture.env.mock_all_auths();
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );

    // This should fail with an unauthorized error
    fixture.registry.add_support_for_asset(
        &fixture.user,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );
}

#[test]
fn test_remove_support_for_asset() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();
    let asset = fixture.create_asset();

    // Mock authorization for the admin
    fixture.env.mock_all_auths();

    // Register adapter and add asset support first
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );
    fixture.registry.add_support_for_asset(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );
    assert!(fixture.verify_asset_supported(protocol.clone(), &asset));

    // Clear events before removing asset support
    let _ = fixture.env.events().all();

    // Remove asset support
    fixture.registry.remove_support_for_asset(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id().clone(),
        &asset,
    );

    let published_event = vec![&fixture.env, fixture.env.events().all().last_unchecked()];
    let expected_event = vec![
        &fixture.env,
        (
            fixture.registry.address.clone(),
            (
                Symbol::new(&fixture.env, "remove_support_for_asset"),
                SupportedYieldType::Lending.id(),
            )
                .into_val(&fixture.env),
            (protocol.id().clone(), asset.clone()).into_val(&fixture.env),
        ),
    ];
    assert_eq!(published_event, expected_event);

    // Verify asset is no longer supported
    assert!(!fixture.verify_asset_supported(protocol.clone(), &asset));
    let is_supported = fixture.registry.is_supported_asset(
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );
    assert!(!is_supported);
}

#[test]
#[should_panic(expected = "Error(Contract, #1301)")]
fn test_remove_support_for_asset_unauthorized() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();
    let asset = fixture.create_asset();

    // Mock authorization for the admin to register adapter and add asset support
    fixture.env.mock_all_auths();
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );
    fixture.registry.add_support_for_asset(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );

    // This should fail with an unauthorized error
    fixture.registry.remove_support_for_asset(
        &fixture.user,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset,
    );
}

// Test checking if an asset is supported
#[test]
fn test_is_supported_asset() {
    let fixture = TestFixture::create();
    let (adapter_address, protocol) = fixture.create_adapter();
    let asset1 = fixture.create_asset();
    let asset2 = fixture.create_asset();

    // Mock authorization for the admin
    fixture.env.mock_all_auths();

    // Register adapter and add support for asset1 only
    fixture.registry.register_adapter(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &adapter_address,
    );
    fixture.registry.add_support_for_asset(
        &fixture.admin,
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset1,
    );

    // Verify asset1 is supported
    let is_supported1 = fixture.registry.is_supported_asset(
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset1,
    );
    assert!(is_supported1);

    // Verify asset2 is not supported
    let is_supported2 = fixture.registry.is_supported_asset(
        &SupportedYieldType::Lending.id(),
        &protocol.id(),
        &asset2,
    );
    assert!(!is_supported2);

    // Verify non-existent protocol returns false for supported assets
    let non_existent_protocol = SupportedAdapter::Custom(symbol_short!("SIMP")); // We're using the same enum but treating it as a different protocol
    let is_supported_non_existent = fixture.registry.is_supported_asset(
        &SupportedYieldType::Lending.id(),
        &non_existent_protocol.id(),
        &asset1,
    );
    assert!(!is_supported_non_existent);
}
