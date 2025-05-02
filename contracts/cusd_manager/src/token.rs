use soroban_sdk::{Address, Env, token};


pub fn process_token_mint(e: &Env, to: Address, token_address: Address, amount: i128) { 
    let token_client= token::StellarAssetClient::new(&e, &token_address);
    token_client.mint(&to, &amount);
}

pub fn process_token_burn(e: &Env, spender: Address, from: Address, token_address: Address, amount: i128) {
    let token_client= token::TokenClient::new(&e, &token_address);
    token_client.burn_from(&spender, &from, &amount);
}