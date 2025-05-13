// examples/withdraw-collateral.ts
import { Keypair, SorobanRpc } from '@stellar/stellar-sdk';
import { Client as LendingYieldControllerClient } from '../ts/lending_yield_controller/dist/index.js';
import { Client as TokenClient } from '@stellar/stellar-sdk/contract';

// REQUIRES INSTALLATION OF THE ts/lending_yield_controller package
async function withdrawCollateral() {
  // Configuration
  const rpcUrl = 'https://soroban-testnet.stellar.org';
  const networkPassphrase = 'Test SDF Network ; September 2015';
  
  // Contract addresses (replace with actual deployed addresses)
  const LENDING_YIELD_CONTROLLER_ADDRESS = 'CBT3CXNXEXDRMNSLA2HQKKEWHR5TEOGYUMPA3UXQXCU2VXQQ67B2MZML';
  const USDC_TOKEN_ADDRESS = 'CDHHR356G725HNLAAQ74WBGVT6Y6ZFZLM2TIHLDCOZTJ2SVZ7P3EANYT';
  
  // User keypair (replace with your own or load from secure storage)
  const userKeypair = Keypair.random(); // In production, load your keypair securely
  const userPublicKey = userKeypair.publicKey();
  
  // Initialize RPC server
  const server = new SorobanRpc.Server(rpcUrl);
  
  try {
    console.log('🏦 Starting collateral withdrawal process...');
    
    // Step 1: Initialize the lending yield controller client
    const lendingClient = new LendingYieldControllerClient({
      contractId: LENDING_YIELD_CONTROLLER_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Step 2: Get cUSD manager to check cUSD balance
    const cusdManagerAddress = await lendingClient.get_cusd_manager();
    console.log(`📍 cUSD Manager: ${cusdManagerAddress.result}`);
    
    // Step 3: Check user's cUSD balance
    console.log('💰 Checking cUSD balance...');
    const cusdClient = new TokenClient({
      contractId: cusdManagerAddress.result,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    const cusdBalanceResult = await cusdClient.balance({ id: userPublicKey });
    const cusdBalance = cusdBalanceResult.result;
    console.log(`Current cUSD balance: ${Number(cusdBalance) / 10_000_000} cUSD`);
    
    if (Number(cusdBalance) <= 0) {
      throw new Error('No cUSD balance to withdraw collateral against');
    }
    
    // Step 4: Define withdrawal amount (should not exceed cUSD balance)
    const withdrawAmount = BigInt(50 * 10_000_000); // 50 USDC worth in smallest unit
    
    if (withdrawAmount > BigInt(cusdBalance)) {
      throw new Error(
        `Withdrawal amount (${Number(withdrawAmount) / 10_000_000}) exceeds cUSD balance (${Number(cusdBalance) / 10_000_000})`
      );
    }
    
    // Step 5: Check current allowance for cUSD spending
    console.log('🔍 Checking cUSD allowance...');
    const allowanceResult = await cusdClient.allowance({
      from: userPublicKey,
      spender: LENDING_YIELD_CONTROLLER_ADDRESS,
    });
    const currentAllowance = allowanceResult.result;
    
    // Step 6: Approve cUSD spending if needed
    if (BigInt(currentAllowance) < withdrawAmount) {
      console.log('✅ Approving cUSD spending...');
      const approveTransaction = await cusdClient.approve({
        from: userPublicKey,
        spender: LENDING_YIELD_CONTROLLER_ADDRESS,
        amount: withdrawAmount,
        expiration_ledger: 0, // No expiration
      });
      
      // Sign and submit the approval transaction
      const signedApproval = await approveTransaction.signAuthEntries({
        publicKey: userPublicKey,
        signAuthEntry: (auth) => userKeypair.sign(auth.toXDR()).toString('base64'),
      });
      
      const approvalResponse = await signedApproval.signAndSend({ 
        signTransaction: (tx) => userKeypair.sign(tx.toXDR()).toString('base64'),
      });
      
      console.log(`Approval transaction hash: ${approvalResponse.hash}`);
      
      // Wait for approval confirmation
      let approvalStatus = await server.getTransaction(approvalResponse.hash);
      while (approvalStatus.status === 'PENDING') {
        await new Promise(resolve => setTimeout(resolve, 2000));
        approvalStatus = await server.getTransaction(approvalResponse.hash);
      }
      
      if (approvalStatus.status !== 'SUCCESS') {
        throw new Error(`Approval failed: ${approvalStatus.status}`);
      }
      
      console.log('✅ cUSD approval successful!');
    }
    
    // Step 7: Withdraw collateral
    console.log('🏃‍♂️ Withdrawing collateral...');
    const withdrawTransaction = await lendingClient.withdraw_collateral({
      protocol: 'BC_LA', // Blend Capital Lending Adapter
      user: userPublicKey,
      asset: USDC_TOKEN_ADDRESS,
      amount: withdrawAmount,
    });
    
    // Sign and submit the withdrawal transaction
    const signedWithdraw = await withdrawTransaction.signAuthEntries({
      publicKey: userPublicKey,
      signAuthEntry: (auth) => userKeypair.sign(auth.toXDR()).toString('base64'),
    });
    
    const withdrawResponse = await signedWithdraw.signAndSend({
      signTransaction: (tx) => userKeypair.sign(tx.toXDR()).toString('base64'),
    });
    
    console.log(`Withdrawal transaction hash: ${withdrawResponse.hash}`);
    
    // Step 8: Wait for withdrawal confirmation
    let withdrawStatus = await server.getTransaction(withdrawResponse.hash);
    while (withdrawStatus.status === 'PENDING') {
      await new Promise(resolve => setTimeout(resolve, 2000));
      withdrawStatus = await server.getTransaction(withdrawResponse.hash);
      console.log('⏳ Waiting for withdrawal confirmation...');
    }
    
    if (withdrawStatus.status === 'SUCCESS') {
      console.log('🎉 Collateral withdrawn successfully!');
      
      // Step 9: Check balances after withdrawal
      console.log('\n📊 Post-withdrawal balances:');
      
      // Check USDC balance
      const usdcClient = new TokenClient({
        contractId: USDC_TOKEN_ADDRESS,
        networkPassphrase,
        rpcUrl,
        publicKey: userPublicKey,
      });
      
      const newUsdcBalance = await usdcClient.balance({ id: userPublicKey });
      console.log(`USDC balance: ${Number(newUsdcBalance.result) / 10_000_000} USDC`);
      
      // Check cUSD balance
      const newCusdBalance = await cusdClient.balance({ id: userPublicKey });
      console.log(`cUSD balance: ${Number(newCusdBalance.result) / 10_000_000} cUSD`);
      
      console.log(`\n✅ Successfully withdrew ${Number(withdrawAmount) / 10_000_000} USDC`);
      console.log(`🔥 Burned ${Number(withdrawAmount) / 10_000_000} cUSD`);
      
      return {
        success: true,
        transactionHash: withdrawResponse.hash,
        withdrawnAmount: Number(withdrawAmount),
        newUsdcBalance: Number(newUsdcBalance.result),
        newCusdBalance: Number(newCusdBalance.result),
      };
      
    } else {
      console.error(`❌ Withdrawal failed with status: ${withdrawStatus.status}`);
      
      // Log more details about the failure
      if (withdrawStatus.resultMetaXdr) {
        console.error('Transaction result meta:', withdrawStatus.resultMetaXdr);
      }
      
      throw new Error(`Withdrawal transaction failed: ${withdrawStatus.status}`);
    }
    
  } catch (error) {
    console.error('❌ Error during withdrawal:', error);
    
    // Handle specific error cases
    if (error.message?.includes('Insufficient balance')) {
      console.error('💡 Make sure you have enough cUSD to burn for the withdrawal');
    } else if (error.message?.includes('Asset is not supported')) {
      console.error('💡 The asset you are trying to withdraw is not supported by the adapter');
    }
    
    throw error;
  }
}

// Function to check withdrawal feasibility
async function checkWithdrawalFeasibility(withdrawAmount: bigint) {
  const rpcUrl = 'https://soroban-testnet.stellar.org';
  const networkPassphrase = 'Test SDF Network ; September 2015';
  const LENDING_YIELD_CONTROLLER_ADDRESS = 'CBT3CXNXEXDRMNSLA2HQKKEWHR5TEOGYUMPA3UXQXCU2VXQQ67B2MZML';
  
  const userKeypair = Keypair.random();
  const userPublicKey = userKeypair.publicKey();
  
  try {
    console.log('🔍 Checking withdrawal feasibility...');
    
    const lendingClient = new LendingYieldControllerClient({
      contractId: LENDING_YIELD_CONTROLLER_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Get cUSD manager and check balance
    const cusdManagerAddress = await lendingClient.get_cusd_manager();
    const cusdClient = new TokenClient({
      contractId: cusdManagerAddress.result,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    const cusdBalance = await cusdClient.balance({ id: userPublicKey });
    const balance = Number(cusdBalance.result);
    
    const feasible = balance >= Number(withdrawAmount);
    
    console.log('\n📋 Withdrawal Feasibility Check:');
    console.log(`- Current cUSD Balance: ${balance / 10_000_000} cUSD`);
    console.log(`- Desired Withdrawal: ${Number(withdrawAmount) / 10_000_000} USDC`);
    console.log(`- Feasible: ${feasible ? '✅' : '❌'}`);
    
    if (!feasible) {
      const shortfall = Number(withdrawAmount) - balance;
      console.log(`- Shortfall: ${shortfall / 10_000_000} cUSD`);
    }
    
    return {
      feasible,
      currentBalance: balance,
      withdrawAmount: Number(withdrawAmount),
      shortfall: feasible ? 0 : Number(withdrawAmount) - balance,
    };
    
  } catch (error) {
    console.error('❌ Error checking withdrawal feasibility:', error);
    throw error;
  }
}

// Run the function if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  async function main() {
    const withdrawAmount = BigInt(50 * 10_000_000); // 50 USDC
    
    // First check feasibility
    await checkWithdrawalFeasibility(withdrawAmount);
    
    console.log('\n' + '='.repeat(50) + '\n');
    
    // Then attempt withdrawal
    await withdrawCollateral();
  }
  
  main()
    .then(() => console.log('✅ Withdrawal process completed successfully'))
    .catch((error) => {
      console.error('❌ Withdrawal process failed:', error);
      process.exit(1);
    });
}

export { withdrawCollateral, checkWithdrawalFeasibility };