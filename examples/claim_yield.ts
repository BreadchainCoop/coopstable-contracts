// examples/claim-yield.ts
import { Keypair, SorobanRpc } from '@stellar/stellar-sdk';
import { Client as LendingYieldControllerClient } from '../ts/lending_yield_controller/dist/index.js';
import { Client as YieldDistributorClient } from '../ts/yield_distributor/dist/index.js';

// REQUIRES INSTALLATION OF THE ts/lending_yield_controller package
async function claimYield() {
  // Configuration
  const rpcUrl = 'https://soroban-testnet.stellar.org';
  const networkPassphrase = 'Test SDF Network ; September 2015';
  
  // Contract addresses (replace with actual deployed addresses)
  const LENDING_YIELD_CONTROLLER_ADDRESS = 'CBT3CXNXEXDRMNSLA2HQKKEWHR5TEOGYUMPA3UXQXCU2VXQQ67B2MZML';
  
  // Admin keypair (only admin can claim yield)
  const adminKeypair = Keypair.random(); // In production, load the admin keypair securely
  const adminPublicKey = adminKeypair.publicKey();
  
  // Initialize RPC server
  const server = new SorobanRpc.Server(rpcUrl);
  
  try {
    console.log('💎 Starting yield claiming process...');
    
    // Step 1: Initialize the lending yield controller client
    const lendingClient = new LendingYieldControllerClient({
      contractId: LENDING_YIELD_CONTROLLER_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: adminPublicKey,
    });
    
    // Step 2: Check current available yield
    console.log('📊 Checking available yield...');
    const yieldResult = await lendingClient.get_yield();
    const availableYield = yieldResult.result;
    
    console.log(`Available yield: ${Number(availableYield) / 10_000_000} tokens`);
    
    if (Number(availableYield) <= 0) {
      console.log('⚠️  No yield available to claim at this time');
      return {
        success: false,
        reason: 'No yield available',
        availableYield: 0,
      };
    }
    
    // Step 3: Get the yield distributor to check distribution availability
    const distributorAddress = await lendingClient.get_yield_distributor();
    console.log(`📍 Yield distributor: ${distributorAddress.result}`);
    
    // Step 4: Check if distribution is available
    const distributorClient = new YieldDistributorClient({
      contractId: distributorAddress.result,
      networkPassphrase,
      rpcUrl,
      publicKey: adminPublicKey,
    });
    
    const isDistributionAvailable = await distributorClient.is_distribution_available();
    console.log(`🕒 Distribution available: ${isDistributionAvailable.result}`);
    
    if (!isDistributionAvailable.result) {
      const nextDistributionTime = await distributorClient.get_next_distribution_time();
      console.log(`⏳ Next distribution available at: ${nextDistributionTime.result}`);
      console.log('⚠️  Distribution period has not elapsed yet');
      
      return {
        success: false,
        reason: 'Distribution period not elapsed',
        nextDistributionTime: nextDistributionTime.result,
      };
    }
    
    // Step 5: Check that there are members to receive distribution
    const members = await distributorClient.list_members();
    console.log(`👥 Distribution members: ${members.result.length}`);
    
    if (members.result.length === 0) {
      console.log('⚠️  No members registered for yield distribution');
      return {
        success: false,
        reason: 'No distribution members',
        availableYield: Number(availableYield),
      };
    }
    
    // Step 6: Claim the yield
    console.log('🎯 Claiming yield...');
    const claimTransaction = await lendingClient.claim_yield();
    
    // Sign and submit the claim transaction
    const signedClaim = await claimTransaction.signAuthEntries({
      publicKey: adminPublicKey,
      signAuthEntry: (auth) => adminKeypair.sign(auth.toXDR()).toString('base64'),
    });
    
    const claimResponse = await signedClaim.signAndSend({
      signTransaction: (tx) => adminKeypair.sign(tx.toXDR()).toString('base64'),
    });
    
    console.log(`Claim transaction hash: ${claimResponse.hash}`);
    
    // Step 7: Wait for claim confirmation
    let claimStatus = await server.getTransaction(claimResponse.hash);
    while (claimStatus.status === 'PENDING') {
      await new Promise(resolve => setTimeout(resolve, 2000));
      claimStatus = await server.getTransaction(claimResponse.hash);
      console.log('⏳ Waiting for transaction confirmation...');
    }
    
    if (claimStatus.status === 'SUCCESS') {
      console.log('🎉 Yield claimed successfully!');
      
      // Step 8: Get the claimed amount from the transaction result
      const claimedAmount = claimTransaction.result;
      console.log(`💰 Claimed yield amount: ${Number(claimedAmount) / 10_000_000} tokens`);
      
      // Step 9: Get distribution details
      const treasuryShare = await distributorClient.get_treasury_share();
      const treasury = await distributorClient.get_treasury();
      
      const treasuryPercentage = Number(treasuryShare.result) / 100;
      const treasuryAmount = (Number(claimedAmount) * treasuryPercentage) / 100;
      const memberAmount = Number(claimedAmount) - treasuryAmount;
      const perMemberAmount = memberAmount / members.result.length;
      
      console.log('\n📊 Distribution Details:');
      console.log(`- Total Claimed: ${Number(claimedAmount) / 10_000_000} tokens`);
      console.log(`- Treasury Share: ${treasuryPercentage}%`);
      console.log(`- Treasury Amount: ${treasuryAmount / 10_000_000} tokens`);
      console.log(`- Member Share: ${memberAmount / 10_000_000} tokens`);
      console.log(`- Per Member: ${perMemberAmount / 10_000_000} tokens`);
      console.log(`- Number of Members: ${members.result.length}`);
      
      return {
        success: true,
        transactionHash: claimResponse.hash,
        claimedAmount: Number(claimedAmount),
        distributionDetails: {
          totalClaimed: Number(claimedAmount),
          treasuryAmount,
          memberAmount,
          perMemberAmount,
          memberCount: members.result.length,
        },
      };
      
    } else {
      console.error(`❌ Claim failed with status: ${claimStatus.status}`);
      
      // Log more details about the failure
      if (claimStatus.resultMetaXdr) {
        console.error('Transaction result meta:', claimStatus.resultMetaXdr);
      }
      
      throw new Error(`Claim transaction failed: ${claimStatus.status}`);
    }
    
  } catch (error) {
    console.error('❌ Error during yield claiming:', error);
    
    // Handle specific error cases
    if (error.message?.includes('Distribution not ready yet')) {
      console.error('💡 The distribution period has not elapsed since the last claim');
    } else if (error.message?.includes('no admin')) {
      console.error('💡 Only admin can claim yield. Make sure you are using the admin keypair');
    }
    
    throw error;
  }
}

// Function to check yield claim status and distribution readiness
async function checkClaimStatus() {
  const rpcUrl = 'https://soroban-testnet.stellar.org';
  const networkPassphrase = 'Test SDF Network ; September 2015';
  const LENDING_YIELD_CONTROLLER_ADDRESS = 'CBT3CXNXEXDRMNSLA2HQKKEWHR5TEOGYUMPA3UXQXCU2VXQQ67B2MZML';
  
  const userKeypair = Keypair.random();
  const userPublicKey = userKeypair.publicKey();
  
  try {
    console.log('🔍 Checking yield claim status...');
    
    const lendingClient = new LendingYieldControllerClient({
      contractId: LENDING_YIELD_CONTROLLER_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Get current yield
    const yield_result = await lendingClient.get_yield();
    const availableYield = yield_result.result;
    
    // Get distributor info
    const distributorAddress = await lendingClient.get_yield_distributor();
    const distributorClient = new YieldDistributorClient({
      contractId: distributorAddress.result,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Check distribution availability
    const isDistributionAvailable = await distributorClient.is_distribution_available();
    const nextDistributionTime = await distributorClient.get_next_distribution_time();
    const distributionPeriod = await distributorClient.get_distribution_period();
    
    console.log('\n📋 Claim Status Summary:');
    console.log(`- Available Yield: ${Number(availableYield) / 10_000_000} tokens`);
    console.log(`- Distribution Ready: ${isDistributionAvailable.result ? '✅' : '❌'}`);
    console.log(`- Next Distribution Time: ${nextDistributionTime.result}`);
    console.log(`- Distribution Period: ${distributionPeriod.result} seconds`);
    
    return {
      availableYield: Number(availableYield),
      isDistributionAvailable: isDistributionAvailable.result,
      nextDistributionTime: nextDistributionTime.result,
      distributionPeriod: distributionPeriod.result,
    };
    
  } catch (error) {
    console.error('❌ Error checking claim status:', error);
    throw error;
  }
}

// Run the function if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  async function main() {
    // First check the status
    await checkClaimStatus();
    
    console.log('\n' + '='.repeat(50) + '\n');
    
    // Then attempt to claim
    await claimYield();
  }
  
  main()
    .then(() => console.log('✅ Claim process completed successfully'))
    .catch((error) => {
      console.error('❌ Claim process failed:', error);
      process.exit(1);
    });
}

export { claimYield, checkClaimStatus };