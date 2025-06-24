use soroban_sdk::{Address, Env, Symbol};

pub struct CUSDEvents {}

impl CUSDEvents {
    pub fn set_cusd_manager(e: &Env, new_manager: Address) {
        let topics: (Symbol,) = (Symbol::new(e, "set_cud_manager"),);
        e.events().publish(topics, new_manager);
    }
    pub fn set_admin(e: &Env, new_admin: Address) {
        let topics: (Symbol,) = (Symbol::new(e, "set_admin"),);
        e.events().publish(topics, new_admin);
    }
}
