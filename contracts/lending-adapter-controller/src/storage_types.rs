use soroban_sdk::{contracttype, Address, Env, Map, Symbol};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const REGISTRY_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const REGISTRY_LIFETIME_THRESHOLD: u32 = REGISTRY_BUMP_AMOUNT - DAY_IN_LEDGERS;

// Data keys for contract storage
#[contracttype]
pub enum DataKey {
    LendingAdapterRegistry,
}

#[contracttype]
pub enum LendingAdapter {
    BlendLendingAdapter
} 

#[derive(Clone)]
#[contracttype]
pub struct LendingAdapterRegistry {
    map: Map<Symbol, Address>
}

impl LendingAdapterRegistry {
    pub fn new(env: &Env) -> Self {
        Self {
            map: Map::new(env)
        }
    }
    
    pub fn contains_key(&self, key: Symbol) -> bool {
        self.map.contains_key(key)
    }
    
    pub fn contains_value(&self, value: Address) -> bool {
        self.map.keys().iter().any(|key| self.map.get(key) == Some(value.clone()))
    }
    
    pub fn set(&mut self, key: Symbol, value: Address) {
        self.map.set(key, value);
    }
    
    pub fn remove(&mut self, key: Symbol) {
        self.map.remove(key);
    }
}

