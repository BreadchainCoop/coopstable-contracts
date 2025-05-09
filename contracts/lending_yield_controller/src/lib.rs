#![no_std]
pub mod cusd_manager {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/cusd_manager.wasm"
    );
}

pub mod yield_adapter_registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/yield_adapter_registry.wasm"
    );
}

pub mod yield_distributor {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/yield_distributor.wasm"
    );
}

// pub mod yield_adapter {
//     soroban_sdk::contractimport!(
//         file = "../../target/wasm32-unknown-unknown/release/deps/yield_adapter.wasm"
//     );
// }

mod contract;
mod constants;
mod events;
mod test;