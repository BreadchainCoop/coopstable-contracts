use soroban_sdk::{auth, vec, Address, Env, IntoVal, Symbol, Val, Vec};
use crate::storage;  
use crate::cusd::Client as CUSDTokenClient;

pub fn process_token_mint(e: &Env, to: Address, amount: i128) {
    
    let token_client = CUSDTokenClient::new(&e, &&storage::read_cusd_id(&e));
    let mint_args: Vec<Val> = vec![
        e,
        (&e.current_contract_address()).into_val(e),
        (&to).into_val(e),
        (&amount).into_val(e),
    ];
    e.authorize_as_current_contract(vec![
        &e,
        auth::InvokerContractAuthEntry::Contract(auth::SubContractInvocation {
            context: auth::ContractContext {
                contract: storage::read_cusd_id(e).clone(),
                fn_name: Symbol::new(e, "mint"),
                args: mint_args,
            },
            sub_invocations: vec![&e],
        }),
    ]);
    token_client.mint(&to, &amount);
}

pub fn process_token_burn(
    e: &Env,
    from: Address,
    amount: i128,
) {
    let token_client = CUSDTokenClient::new(&e, &&storage::read_cusd_id(&e));
    token_client.burn(&from, &amount);
}