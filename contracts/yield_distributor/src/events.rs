use soroban_sdk::{Address, Env, Symbol, Vec};

pub struct YieldDistributorEvents {}

impl YieldDistributorEvents {
    pub fn add_member(e: &Env, member: Address) {
        let topics = (Symbol::new(e, "add_member"),);
        e.events().publish(topics, member);
    }

    pub fn remove_member(e: &Env, member: Address) {
        let topics = (Symbol::new(e, "remove_member"),);
        e.events().publish(topics, member);
    }

    pub fn set_treasury(e: &Env, treasury: Address) {
        let topics = (Symbol::new(e, "set_treasury"),);
        e.events().publish(topics, treasury);
    }

    pub fn set_treasury_share(e: &Env, share_bps: u32) {
        let topics = (Symbol::new(e, "set_treasury_share"),);
        e.events().publish(topics, share_bps);
    }

    pub fn set_distribution_period(e: &Env, period: u64) {
        let topics = (Symbol::new(e, "set_distribution_period"),);
        e.events().publish(topics, period);
    }
    
    pub fn set_yield_controller(e: &Env, yield_controller: Address) {
        let topics = (Symbol::new(e, "set_yield_controller"),);
        e.events().publish(topics, yield_controller);
    }

    pub fn distribute_yield(
        e: &Env,
        asset: Address,
        total_amount: i128,
        treasury_amount: i128,
        members: Vec<Address>,
        per_member_amount: i128,
    ) {
        let topics = (Symbol::new(e, "distribute_yield"), asset);
        e.events().publish(
            topics,
            (total_amount, treasury_amount, members, per_member_amount),
        );
    }
}
