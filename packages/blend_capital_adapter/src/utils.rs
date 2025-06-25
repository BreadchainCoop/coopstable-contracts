use soroban_sdk::{Env, Symbol, auth, vec, Val, Vec, Address};

pub fn authenticate_contract(e: &Env, contract: Address, fn_name: Symbol, args: Vec<Val>) {
    e.authorize_as_current_contract(vec![
        &e,
        auth::InvokerContractAuthEntry::Contract(auth::SubContractInvocation {
            context: auth::ContractContext {
                contract: contract,
                fn_name: fn_name,
                args: args,
            },
            sub_invocations: vec![&e],
        }),
    ]);
}