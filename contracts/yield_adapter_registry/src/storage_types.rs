use soroban_sdk::{
    contracttype, 
    Address, 
    Env, 
    Map, 
    Symbol, 
    Vec 
};

pub(crate) const DAY_IN_LEDGERS: u32 = 17280;
pub(crate) const REGISTRY_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
pub(crate) const REGISTRY_LIFETIME_THRESHOLD: u32 = REGISTRY_BUMP_AMOUNT - DAY_IN_LEDGERS;

#[derive(Clone)]
#[contracttype]
pub struct YieldAdapterRegistryMap {
    pub yield_type: Symbol,
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

    pub fn new(env: &Env, yield_type: Symbol) -> Self {
        Self {
            yield_type,
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

    pub fn adapters(&self) -> Vec<Address> {
        let env = self.registry_map.env();
        
        let mut adapter_addresses = Vec::new(env);
        
        for (_protocol_id, adapter_address) in self.registry_map.iter() {
            adapter_addresses.push_back(adapter_address);
        }
        
        adapter_addresses
    }
    
    pub fn adapter_with_assets(&self) -> Vec<(Address, Vec<Address>)> {
    
        let env = self.registry_map.env();
        
        let mut result = Vec::new(env);
        
        for (protocol_id, adapter_address) in self.registry_map.iter() {
            let mut supported_assets = Vec::new(env);
            
            if let Some(asset_map) = self.supported_assets.get(protocol_id.clone()) {

                for (asset_address, is_supported) in asset_map.iter() {

                    if is_supported {
                        supported_assets.push_back(asset_address);
                    }
                }
            }
            
            result.push_back((adapter_address, supported_assets));
        }
        
        result
    }
}
