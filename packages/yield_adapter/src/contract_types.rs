use soroban_sdk::{contracttype, Symbol, symbol_short};

#[derive(Clone)]
#[contracttype]
pub enum SupportedAdapter {
    BlendCapital,
    Custom(Symbol)
}

impl SupportedAdapter {
    pub fn id(&self) -> Symbol {
        match self {
            SupportedAdapter::BlendCapital => symbol_short!("BC_LA"),
            SupportedAdapter::Custom(s) => s.clone(),
        }
    }
}