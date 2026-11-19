use soroban_sdk::{Address, BytesN, Env, Symbol};

pub struct CUSDManagerEvents {}

impl CUSDManagerEvents {
    pub fn issue_cusd(e: &Env, to: Address, amount: i128) {
        let topics = (Symbol::new(e, "issue_cusd"),);
        e.events().publish(topics, (to, amount));
    }

    pub fn set_cusd_id(e: &Env, new_cusd_id: Address) {
        let topics = (Symbol::new(e, "set_cusd_id"),);
        e.events().publish(topics, new_cusd_id);
    }
    
    pub fn set_cusd_issuer(e: &Env, new_issuer: Address) {
        let topics = (Symbol::new(e, "set_cusd_issuer"),);
        e.events().publish(topics, new_issuer);
    }

    pub fn burn_cusd(e: &Env, from: Address, amount: i128) {
        let topics = (Symbol::new(e, "burn_cusd"),);
        e.events().publish(topics, (from, amount));
    }
    
    pub fn set_admin(e: &Env, new_admin: Address) {
        let topics = (Symbol::new(e, "set_admin"),);
        e.events().publish(topics, new_admin);
    }
    
    pub fn set_yield_controller(e: &Env, yield_controller: Address) {
        let topics = (Symbol::new(e, "set_yield_controller"),);
        e.events().publish(topics, yield_controller);
    }

    pub fn upgrade(e: &Env, new_wasm_hash: BytesN<32>) {
        let topics = (Symbol::new(e, "upgrade"),);
        e.events().publish(topics, new_wasm_hash);
    }
}
