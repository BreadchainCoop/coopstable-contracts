#![no_std]
pub mod cusd_manager {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/cusd_manager.wasm",
    );
}

pub mod yield_adapter_registry {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/yield_adapter_registry.wasm"
    );
}

pub mod yield_distributor {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/yield_distributor.wasm"
    );
}

mod storage;
mod storage_types;
mod contract;
mod events;
mod error;
mod controls;
mod test;
mod utils;