**GOAL:** Fix ResourceLimitExceeded error in `make test-claim-yield` by optimizing resource usage while preserving all necessary cross-contract authorization calls.

**IMPLEMENTATION LOOP:** Follow this exact sequence until the goal is achieved:

1. `make test-withdraw` (continue regardless of failure)  
2.  Implement fix (detailed below)
3. `make build-optimized`
4. `make redeploy-protocol` 
5. `make test-deposit`
6. Wait 30 seconds for yield accumulation
7. `make test-claim-yield`
   - If successful: STOP - goal achieved
   - If ResourceLimitExceeded error: restart loop

**CRITICAL OPTIMIZATION AREAS:**

## Phase 1: Contract Client Caching & Reuse

**Problem:** Multiple `LendingAdapterClient::new()` instantiations in loops
**File:** `contracts/lending_yield_controller/src/controls.rs`

In `process_claim_and_distribute_yield` and `process_claim_yield`:
- Cache adapter clients outside loops instead of recreating them
- Store registry client as local variable to avoid repeated calls
- Reuse the same pool clients where possible

```rust
// Before: Creating new clients in each iteration
let adapter_client = LendingAdapterClient::new(e, &adapter_address);

// After: Cache clients outside loops
let mut adapter_clients: Map<Address, LendingAdapterClient> = Map::new(e);
```

## Phase 2: Storage Operation Optimization

**Problem:** Repeated storage reads/writes during yield processing
**File:** `packages/blend_capital_adapter/src/storage.rs`

- Batch storage operations where possible
- Use temporary variables to avoid repeated contract state reads
- Minimize persistent storage writes during yield claiming
- Optimize epoch principal updates to reduce storage operations

## Phase 3: Loop Structure Optimization

**Problem:** Complex nested iterations over adapters and assets
**File:** `contracts/lending_yield_controller/src/controls.rs`

In `process_claim_and_distribute_yield`:
- Pre-filter adapters with available yield to reduce iterations
- Break early from loops when no yield is available
- Reduce the complexity of nested loops over `lend_protocols_with_assets`

```rust
// Optimize this pattern:
for (adapter_address, supported_assets) in lend_protocols_with_assets.iter() {
    for asset in supported_assets.iter() {
        // Multiple operations per asset
    }
}
```

## Phase 4: Memory Usage Reduction

**Problem:** Large data structures consuming memory resources
**Files:** All storage and contract files

- Review `Vec` usage in storage operations
- Use fixed-size arrays where possible instead of dynamic vectors
- Minimize temporary data structure creation
- Optimize the `AssetEpochPrincipal` structure size if possible

## Phase 5: Transaction Footprint Minimization

**Problem:** Too many ledger entry accesses in single transaction
**File:** `contracts/lending_yield_controller/src/controls.rs`

- Reduce the number of distinct ledger entries accessed per transaction
- Combine related storage operations
- Consider processing fewer assets per transaction if necessary
- Optimize the footprint by reducing cross-contract state access

## Phase 6: Algorithm Efficiency Improvements

**Problem:** Inefficient yield calculation and processing algorithms

**In `process_claim_yield`:**
- Calculate yield amounts upfront to avoid repeated adapter calls
- Skip processing assets with zero yield early
- Optimize the yield → deposit → distribute flow

**In `read_yield`:**
- Cache yield calculations to avoid repeated complex operations
- Pre-compute totals where possible

## Phase 7: Event and Return Value Optimization

**Problem:** Large events or return values consuming bandwidth resources

- Minimize event data size in `LendingYieldControllerEvents`
- Reduce return value sizes where possible
- Optimize logging and event emission

**SPECIFIC IMPLEMENTATION GUIDANCE:**

### Key File: `contracts/lending_yield_controller/src/controls.rs`

**Optimize `process_claim_and_distribute_yield`:**
```rust
pub fn process_claim_and_distribute_yield(e: &Env) -> i128 {
    let registry_client = storage::adapter_registry_client(e);
    let lend_protocols_with_assets = registry_client.get_adapters_with_assets(&storage_types::YIELD_TYPE.id());
    
    // Pre-filter adapters with actual yield to reduce processing
    let mut claimed_total: i128 = 0;
    
    // Cache commonly used clients
    let distributor = storage::distributor_client(e);
    let cusd_manager = storage::cusd_manager_client(e);
    
    // Process in batches or optimize the loop structure
    for (adapter_address, supported_assets) in lend_protocols_with_assets.iter() {
        // Check if adapter has any yield before processing
        let adapter = LendingAdapterClient::new(e, &adapter_address);
        
        // Early continue if no yield available
        let has_yield = supported_assets.iter().any(|asset| adapter.get_yield(&asset) > 0);
        if !has_yield {
            continue;
        }
        
        claimed_total += process_claim_yield(e, &adapter_address, supported_assets.clone());
    }
    
    claimed_total
}
```

### Key File: `packages/blend_capital_adapter/src/storage.rs`

**Optimize storage operations:**
- Batch `extend_ttl` calls where possible
- Minimize instance storage usage
- Use more efficient data structures for epoch tracking

**PRESERVE ALL AUTHORIZATION CALLS:**
- Keep all `utils::authenticate_contract` calls exactly as they are
- Do not modify the authorization flow
- All cross-contract auth must remain intact for security

**SUCCESS CRITERIA:**
- `make test-claim-yield` executes without ResourceLimitExceeded error
- All authorization security preserved
- Yield calculation accuracy maintained
- Distribution functionality works correctly

**DEBUGGING APPROACH:**
- Add temporary logging to identify which specific operation consumes the most resources
- Monitor transaction complexity step by step
- Test each optimization phase individually

**CRITICAL CONSTRAINTS:**
- DO NOT remove or modify any `utils::authenticate_contract` calls
- DO NOT change the authorization patterns
- DO NOT alter the business logic of yield calculation and distribution
- Focus ONLY on algorithmic and data structure optimizations

Implement these optimizations systematically, focusing on reducing computational complexity and memory usage while preserving the complete security model.