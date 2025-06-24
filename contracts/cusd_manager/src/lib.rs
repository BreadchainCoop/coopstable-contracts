#![no_std]
pub mod contract;
mod events;
mod storage_types;
mod storage;
mod test;
pub mod token;
pub mod cusd {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/cusd_token.wasm"
    );
}
mod error;