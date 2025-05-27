# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build Commands
- `make build` - Build all contracts
- `make clean` - Clean all build artifacts
- `stellar contract build` - Build individual contract (run from contract directory)

### Test Commands
- `make test` - Run all tests (builds first)
- `cargo test` - Run tests for individual contract/package
- `cargo test <test_name>` - Run specific test

### Code Quality
- `make fmt` - Format all code
- `cargo fmt --all` - Format code

### TypeScript Bindings
- `./scripts/generate_bindings.sh` - Generate TypeScript bindings for contracts

## Architecture

This is a Soroban smart contract system on Stellar blockchain implementing a decentralized stable coin (cUSD) with yield generation.

### Core Contracts
1. **cUSD Manager** - Manages stable coin minting/burning with collateral tracking
2. **Lending Yield Controller** - Main controller for deposits, withdrawals, and yield claims
3. **Yield Adapter Registry** - Registry for yield protocol adapters (supports multiple protocols)
4. **Yield Distributor** - Distributes yield between treasury and cooperative members

### Key Patterns
- **Adapter Pattern**: Yield protocols integrated via adapters (e.g., Blend Capital)
- **Role-Based Access**: Uses access_control package with roles (DEFAULT_ADMIN_ROLE, CUSD_ADMIN, YIELD_CONTROLLER)
- **Event System**: All major operations emit events for tracking
- **Modular Design**: Contracts communicate through well-defined interfaces

### Testing Approach
- Unit tests in each contract's `test.rs` file
- Test snapshots stored in `test_snapshots/` directories
- Tests use mock environments and simulate contract interactions