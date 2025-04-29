use soroban_sdk::{contracttype, Address, Env, Map, Symbol, symbol_short};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
pub(crate) const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub(crate) const REGISTRY_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const REGISTRY_LIFETIME_THRESHOLD: u32 = REGISTRY_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub (crate) const YIELD_REGISTRY_KEY: Symbol = symbol_short!("YREG");

#[derive(Clone)]
#[contracttype]
pub struct YieldAdapterRegistryMap {
    map: Map<Symbol, Address>
}

impl YieldAdapterRegistryMap {
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
    
    pub fn set_adapter(&mut self, key: Symbol, value: Address) {
        self.map.set(key, value);
    }

    pub fn get_adapter(&self, key: Symbol) -> Option<Address> {
        self.map.get(key)
    }
    
    pub fn remove(&mut self, key: Symbol) {
        self.map.remove(key);
    }
}
