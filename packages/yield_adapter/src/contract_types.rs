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
            SupportedAdapter::BlendCapital => symbol_short!("BC_LA"),
            SupportedAdapter::Aave         => symbol_short!("AA_LA"),
            SupportedAdapter::Compound     => symbol_short!("CO_LA"),
        }
    }
}