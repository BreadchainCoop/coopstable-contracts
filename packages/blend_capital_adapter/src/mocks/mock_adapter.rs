use soroban_sdk::{
    token::TokenClient,
    Address, Env
};


pub struct MockLendingAdapter;

impl MockLendingAdapter {

    pub fn deposit(
        e: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {
        
        // Return the deposited amount
        amount
    }
    
    pub fn withdraw(
        e: &Env,
        user: Address,
        asset: Address,
        amount: i128
    ) -> i128 {

        let token_client = TokenClient::new(e, &asset);
        token_client.transfer(&e.current_contract_address(), &user, &amount);
        // Return the withdrawn amount
        amount
    }
    
    pub fn get_yield(
        e: &Env,
        user: Address,
        asset: Address
    ) -> i128 {
        // Return the mock yield value for this user and asset
        100_000_000
    }

    pub fn claim_yield(
        e: &Env, 
        user: Address,
        asset: Address
    ) -> i128 {
                
        let yield_amount = 100_000_000;
        
        let token_client = TokenClient::new(e, &asset);
        token_client.transfer(&e.current_contract_address(), &user, &yield_amount);

        // Return the claimed yield amount
        yield_amount
    }
}
