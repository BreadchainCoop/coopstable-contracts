#![no_std]
mod contract;
mod contract_types;
mod constants;
mod storage;
mod artifacts {
    pub mod pool {
        soroban_sdk::contractimport!(file = "./artifacts/pool.wasm");
    } 
    pub mod pool_factory {
        soroban_sdk::contractimport!(file = "./artifacts/pool_factory.wasm");
    }
    pub mod backstop {
        soroban_sdk::contractimport!(file = "./artifacts/backstop.wasm");
    }
}
mod blend_pool_mock;
mod test;