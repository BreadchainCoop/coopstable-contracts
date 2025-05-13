// examples/query-yield.ts
import { Keypair, SorobanRpc } from '@stellar/stellar-sdk';
import { Client as LendingYieldControllerClient } from '../ts/lending_yield_controller/dist/index.js';
import { Client as YieldAdapterRegistryClient } from '../ts/yield_adapter_registry/dist/index.js';

// REQUIRES INSTALLATION OF THE ts/lending_yield_controller package
async function queryYield() {
  // Configuration
  const rpcUrl = 'https://soroban-testnet.stellar.org';
  const networkPassphrase = 'Test SDF Network ; September 2015';
  
  // Contract addresses (replace with actual deployed addresses)
  const LENDING_YIELD_CONTROLLER_ADDRESS = 'CBT3CXNXEXDRMNSLA2HQKKEWHR5TEOGYUMPA3UXQXCU2VXQQ67B2MZML';
  const YIELD_ADAPTER_REGISTRY_ADDRESS = 'CBT3CXNXEXDRMNSLA2HQKKEWHR5TEOGYUMPA3UXQXCU2VXQQ67B2MZML';
  
  // User keypair (for querying, we only need the public key)
  const userKeypair = Keypair.random(); // In production, load your keypair
  const userPublicKey = userKeypair.publicKey();
  
  try {
    console.log('📊 Querying yield information...');
    
    // Step 1: Initialize the lending yield controller client
    const lendingClient = new LendingYieldControllerClient({
      contractId: LENDING_YIELD_CONTROLLER_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Step 2: Get the total yield available for the system
    console.log('🔍 Fetching total system yield...');
    const totalYieldResult = await lendingClient.get_yield();
    const totalYield = totalYieldResult.result;
    
    console.log(`💰 Total yield available: ${Number(totalYield) / 10_000_000} tokens`);
    
    // Step 3: Get information about yield distribution
    const distributorAddress = await lendingClient.get_yield_distributor();
    console.log(`📍 Yield distributor address: ${distributorAddress.result}`);
    
    // Step 4: Get the adapter registry to see which protocols are earning yield
    const registryAddress = await lendingClient.get_adapter_registry();
    console.log(`📍 Adapter registry address: ${registryAddress.result}`);
    
    // Step 5: Query the registry for detailed yield adapter information
    const registryClient = new YieldAdapterRegistryClient({
      contractId: registryAddress.result,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Get all adapters and their supported assets for lending
    console.log('🔍 Fetching adapter information...');
    const adaptersWithAssetsResult = await registryClient.get_adapters_with_assets({
      yield_type: 'LEND', // Lending yield type
    });
    
    const adaptersWithAssets = adaptersWithAssetsResult.result;
    
    console.log('\n📋 Active Yield Adapters:');
    for (const [adapterAddress, supportedAssets] of adaptersWithAssets) {
      console.log(`\n  📌 Adapter: ${adapterAddress}`);
      console.log(`  🏷️  Supported Assets:`);
      
      for (const asset of supportedAssets) {
        console.log(`    - ${asset}`);
        
        // Check if this asset is supported
        const isSupported = await registryClient.is_supported_asset({
          yield_type: 'LEND',
          protocol: 'BC_LA', // Blend Capital adapter
          asset_address: asset,
        });
        
        console.log(`      ✅ Supported: ${isSupported.result}`);
      }
    }
    
    // Step 6: Additional system information
    console.log('\n📈 System Overview:');
    console.log(`- Total Available Yield: ${Number(totalYield) / 10_000_000} tokens`);
    console.log(`- Active Adapters: ${adaptersWithAssets.length}`);
    console.log(`- Yield Controller: ${LENDING_YIELD_CONTROLLER_ADDRESS}`);
    console.log(`- Yield Distributor: ${distributorAddress.result}`);
    
    // Step 7: Get CUSD manager info
    const cusdManagerAddress = await lendingClient.get_cusd_manager();
    console.log(`- cUSD Manager: ${cusdManagerAddress.result}`);
    
    return {
      totalYield: Number(totalYield),
      adaptersWithAssets,
      distributorAddress: distributorAddress.result,
      cusdManagerAddress: cusdManagerAddress.result,
    };
    
  } catch (error) {
    console.error('❌ Error querying yield information:', error);
    throw error;
  }
}

async function queryUserSpecificYield(userAddress: string) {
  console.log(`\n👤 Querying yield for specific user: ${userAddress}`);
  
  // Note: To get user-specific yield, you would need to:
  // 1. Query the specific yield adapters directly
  // 2. Calculate yield based on user's deposits
  // 3. Check user's position in the yield distribution system
  
  // This is a simplified example of how you might approach it:
  console.log('💡 Note: User-specific yield queries require:');
  console.log('  - Calling get_yield() on specific adapters with user address');
  console.log('  - Calculating yield based on time and deposit amount');
  console.log('  - Checking membership in yield distribution');
  
  return {
    message: 'User-specific yield calculation requires adapter-specific queries',
    suggestion: 'Use blend_capital_adapter bindings to call get_yield() with user and asset parameters',
  };
}

// Run the function if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  async function main() {
    const yieldInfo = await queryYield();
    console.log('\n✅ Successfully retrieved yield information');
    
    // Example of querying user-specific yield
    await queryUserSpecificYield('GA...'); // Replace with actual user address
  }
  
  main()
    .then(() => console.log('✅ Query process completed successfully'))
    .catch((error) => {
      console.error('❌ Query process failed:', error);
      process.exit(1);
    });
}

export { queryYield, queryUserSpecificYield };