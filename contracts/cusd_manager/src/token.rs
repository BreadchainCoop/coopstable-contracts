use soroban_sdk::{token, Address, Env };
use crate::storage;  

pub fn process_token_mint(e: &Env, to: Address, amount: i128) {    
    let token_client = token::StellarAssetClient::new(&e, &storage::read_cusd_id(&e));
    token_client.mint(&to, &amount);
    storage::increase_cusd_supply(&e, &amount);
}

pub fn process_token_burn(
    e: &Env,
    from: Address,
    amount: i128,
) {
    let token_client = token::TokenClient::new(&e, &&storage::read_cusd_id(&e));
    token_client.burn(&from, &amount);
    storage::decrease_cusd_supply(&e, &amount);
}

pub fn set_issuer(e: &Env, cusd_id: &Address, new_issuer: &Address) {
    let token_client = token::StellarAssetClient::new(&e, &cusd_id);
    token_client.set_admin(&new_issuer);
}

