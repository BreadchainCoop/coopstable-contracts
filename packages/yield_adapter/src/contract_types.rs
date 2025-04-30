use soroban_sdk::{contracttype, Symbol, symbol_short};

#[derive(Clone)]
#[contracttype]
pub enum SupportedAdapter {
    BlendCapital,
    Aave,
    Compound,
}

impl SupportedAdapter {
    pub fn id(&self) -> Symbol {
        match self {
            SupportedAdapter::BlendCapital => symbol_short!("BLA"),
            SupportedAdapter::Aave         => symbol_short!("ALA"),
            SupportedAdapter::Compound     => symbol_short!("CLA"),
        }
    }
}