#![no_std]
mod contract;
mod contract_types;
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
