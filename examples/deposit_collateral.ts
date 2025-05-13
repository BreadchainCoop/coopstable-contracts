import { Keypair, SorobanRpc } from '@stellar/stellar-sdk';
import { Client as LendingYieldControllerClient } from '../ts/lending_yield_controller/dist/index.js';
import { Client as TokenClient } from '@stellar/stellar-sdk/contract';

// REQUIRES INSTALLATION OF THE ts/lending_yield_controller package
async function depositCollateral() {
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
    console.log('🚀 Starting collateral deposit process...');
    
    // Step 1: Initialize the lending yield controller client
    const lendingClient = new LendingYieldControllerClient({
      contractId: LENDING_YIELD_CONTROLLER_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Step 2: Initialize the token client for USDC
    const tokenClient = new TokenClient({
      contractId: USDC_TOKEN_ADDRESS,
      networkPassphrase,
      rpcUrl,
      publicKey: userPublicKey,
    });
    
    // Step 3: Check user's USDC balance
    console.log('📊 Checking USDC balance...');
    const balanceResult = await tokenClient.balance({ id: userPublicKey });
    const balance = balanceResult.result;
    console.log(`Current USDC balance: ${balance}`);
    
    // Step 4: Define deposit amount (in smallest unit, USDC has 7 decimals)
    const depositAmount = BigInt(100 * 10_000_000); // 100 USDC in smallest unit
    
    if (BigInt(balance) < depositAmount) {
      throw new Error(`Insufficient USDC balance. Need ${depositAmount}, have ${balance}`);
    }
    
    // Step 5: Approve the lending controller to spend USDC
    console.log('✅ Approving USDC spending...');
    const approveTransaction = await tokenClient.approve({
      from: userPublicKey,
      spender: LENDING_YIELD_CONTROLLER_ADDRESS,
      amount: depositAmount,
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
    
    // Step 6: Wait for approval confirmation
    let approvalStatus = await server.getTransaction(approvalResponse.hash);
    while (approvalStatus.status === 'PENDING') {
      await new Promise(resolve => setTimeout(resolve, 2000));
      approvalStatus = await server.getTransaction(approvalResponse.hash);
    }
    
    if (approvalStatus.status !== 'SUCCESS') {
      throw new Error(`Approval failed: ${approvalStatus.status}`);
    }
    
    console.log('✅ USDC approval successful!');
    
    // Step 7: Deposit collateral to get cUSD
    console.log('🏦 Depositing collateral...');
    const depositTransaction = await lendingClient.deposit_collateral({
      protocol: 'BC_LA', // Blend Capital Lending Adapter
      user: userPublicKey,
      asset: USDC_TOKEN_ADDRESS,
      amount: depositAmount,
    });
    
    // Sign and submit the deposit transaction
    const signedDeposit = await depositTransaction.signAuthEntries({
      publicKey: userPublicKey,
      signAuthEntry: (auth) => userKeypair.sign(auth.toXDR()).toString('base64'),
    });
    
    const depositResponse = await signedDeposit.signAndSend({
      signTransaction: (tx) => userKeypair.sign(tx.toXDR()).toString('base64'),
    });
    
    console.log(`Deposit transaction hash: ${depositResponse.hash}`);
    
    // Step 8: Wait for deposit confirmation
    let depositStatus = await server.getTransaction(depositResponse.hash);
    while (depositStatus.status === 'PENDING') {
      await new Promise(resolve => setTimeout(resolve, 2000));
      depositStatus = await server.getTransaction(depositResponse.hash);
    }
    
    if (depositStatus.status === 'SUCCESS') {
      console.log('🎉 Collateral deposited successfully!');
      console.log(`You deposited ${Number(depositAmount) / 10_000_000} USDC`);
      console.log(`You received ${Number(depositAmount) / 10_000_000} cUSD in return`);
    } else {
      throw new Error(`Deposit failed: ${depositStatus.status}`);
    }
    
  } catch (error) {
    console.error('❌ Error during deposit:', error);
    throw error;
  }
}

// Run the function if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  depositCollateral()
    .then(() => console.log('✅ Deposit process completed successfully'))
    .catch((error) => {
      console.error('❌ Deposit process failed:', error);
      process.exit(1);
    });
}

export { depositCollateral };