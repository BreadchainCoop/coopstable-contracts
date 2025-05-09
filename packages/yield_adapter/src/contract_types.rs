use soroban_sdk::{contracttype, symbol_short, Symbol};

#[derive(Clone)]
#[contracttype]
pub enum SupportedAdapter {
    BlendCapital,
    Custom(Symbol),
}

impl SupportedAdapter {
    pub fn id(&self) -> Symbol {
        match self {
            SupportedAdapter::BlendCapital => symbol_short!("BC_LA"),
            SupportedAdapter::Custom(s) => s.clone(),
        }
    }
}

#[derive(Clone)]
#[contracttype]
pub enum SupportedYieldType {
    Lending,
    Liquidity,
    Custom(Symbol),
}

impl SupportedYieldType {
    pub fn id(&self) -> Symbol {
        match self {
            SupportedYieldType::Lending => symbol_short!("LEND"),
            SupportedYieldType::Liquidity => symbol_short!("LIQUIDITY"),
            SupportedYieldType::Custom(s) => s.clone(),
        }
    }
}
