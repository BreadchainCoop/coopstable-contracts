use soroban_sdk::{
    contracttype, 
    symbol_short,
    Address, 
    Env, 
    Map, 
    Symbol, 
};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;

pub(crate) const REGISTRY_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const REGISTRY_LIFETIME_THRESHOLD: u32 = REGISTRY_BUMP_AMOUNT - DAY_IN_LEDGERS;

pub (crate) const YIELD_REGISTRY_KEY: Symbol = symbol_short!("YREG");

#[derive(Clone)]
#[contracttype]
pub struct YieldAdapterRegistryMap {
    registry_map: Map<Symbol, Address>,
    supported_assets: Map<Symbol, Map<Address, bool>>
}

impl YieldAdapterRegistryMap {
    fn supported_asset_nested_map(&self, key: Symbol) -> Map<Address, bool> {
        if let Some(map) = self.supported_assets.get(key) {
            map
        } else {
            let map = Map::new(self.supported_assets.env());
            map
        }
    }

    pub fn new(env: &Env) -> Self {
        Self {
            registry_map: Map::new(env),
            supported_assets: Map::new(env)
        }
    }
    
    pub fn contains_key(&self, key: Symbol) -> bool {
        self.registry_map.contains_key(key)
    }
    
    pub fn contains_value(&self, value: Address) -> bool {
        self.registry_map.keys().iter().any(|key| self.registry_map.get(key) == Some(value.clone()))
    }
    
    pub fn set_adapter(&mut self, key: Symbol, value: Address) {
        self.registry_map.set(key, value);
    }

    pub fn get_adapter(&self, key: Symbol) -> Address {
        self.registry_map.get(key.clone()).unwrap()
    }

    pub fn remove(&mut self, key: Symbol) {
        self.registry_map.remove(key);
    }

    pub fn support_asset(&mut self, key: Symbol, asset: Address) {
        let mut nested_map = self.supported_asset_nested_map(key.clone());
        nested_map.set(asset, true);
        self.supported_assets.set(key, nested_map);
    }

    pub fn remove_asset_support(&mut self, key: Symbol, asset: Address) {
        let mut nested_map = self.supported_asset_nested_map(key.clone());
        nested_map.remove(asset);
        self.supported_assets.set(key, nested_map);
    }

    pub fn supports_asset(&self, key: Symbol, asset: Address) -> bool {
        let nested_map = self.supported_asset_nested_map(key.clone());
        nested_map.get(asset).unwrap_or(false)
    }
}
