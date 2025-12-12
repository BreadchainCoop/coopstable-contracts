import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export enum RequestType {
  Supply = 0,
  Withdraw = 1,
  SupplyCollateral = 2,
  WithdrawCollateral = 3,
  Borrow = 4,
  Repay = 5,
  FillUserLiquidationAuction = 6,
  FillBadDebtAuction = 7,
  FillInterestAuction = 8,
  DeleteLiquidationAuction = 9,
}


export interface Request {
  address: string;
  amount: i128;
  request_type: u32;
}


export interface AssetEpochPrincipal {
  deposits_in_epoch: i128;
  epoch: u64;
  last_updated: u64;
  principal: i128;
  withdrawals: i128;
}


export interface PoolConfig {
  bstop_rate: u32;
}

export type SupportedAdapter = {tag: "BlendCapital", values: void} | {tag: "Custom", values: readonly [string]};

export type SupportedYieldType = {tag: "Lending", values: void} | {tag: "Liquidity", values: void} | {tag: "Custom", values: readonly [string]};

export interface Client {
  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a deposit_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_auth: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a withdraw_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_auth: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a get_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_yield: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a claim_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_yield: ({asset, yield_amount}: {asset: string, yield_amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a claim_yield_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_yield_auth: ({asset, amount}: {asset: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a claim_emissions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_emissions: ({to, asset}: {to: string, asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a claim_emissions_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_emissions_auth: ({to, asset}: {to: string, asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a get_emissions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_emissions: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a protocol_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  protocol_token: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_total_deposited transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_deposited: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_balance: ({user, asset}: {user: string, asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_apy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_apy: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a update_epoch_principal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_epoch_principal: ({asset, epoch, principal}: {asset: string, epoch: u64, principal: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a init transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init: ({initial_asset}: {initial_asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<void>>

  /**
   * Construct and simulate a submit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  submit: ({user, _spender, _to, requests}: {user: string, _spender: string, _to: string, requests: Array<Request>}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Positions>>

  /**
   * Construct and simulate a submit_with_allowance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  submit_with_allowance: ({user, _spender, _sender, _requests}: {user: string, _spender: string, _sender: string, _requests: Array<Request>}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Positions>>

  /**
   * Construct and simulate a get_positions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_positions: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Positions>>

  /**
   * Construct and simulate a get_reserve transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_reserve: ({asset}: {asset: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Reserve>>

  /**
   * Construct and simulate a get_reserve_list transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_reserve_list: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_config: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<PoolConfig>>

  /**
   * Construct and simulate a claim transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim: ({_user, _token_ids, _to}: {_user: string, _token_ids: Array<u32>, _to: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a update_b_rate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_b_rate: ({asset, new_rate}: {asset: string, new_rate: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a add_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_yield: ({asset, yield_amount}: {asset: string, yield_amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_user_emissions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_emissions: ({_user, _reserve_token_id}: {_user: string, _reserve_token_id: u32}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Option<UserEmissionData>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {yield_controller, blend_pool_id, blend_token_id}: {yield_controller: string, blend_pool_id: string, blend_token_id: string},
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy({yield_controller, blend_pool_id, blend_token_id}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAMAAAAAAAAAEHlpZWxkX2NvbnRyb2xsZXIAAAATAAAAAAAAAA1ibGVuZF9wb29sX2lkAAAAAAAAEwAAAAAAAAAOYmxlbmRfdG9rZW5faWQAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAAMZGVwb3NpdF9hdXRoAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+gAAAPtAAAAAwAAABMAAAARAAAD6gAAAAA=",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAANd2l0aGRyYXdfYXV0aAAAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAVhc3NldAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPoAAAD7QAAAAMAAAATAAAAEQAAA+oAAAAA",
        "AAAAAAAAAAAAAAAJZ2V0X3lpZWxkAAAAAAAAAQAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAALY2xhaW1feWllbGQAAAAAAgAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAx5aWVsZF9hbW91bnQAAAALAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAAQY2xhaW1feWllbGRfYXV0aAAAAAIAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+gAAAPtAAAAAwAAABMAAAARAAAD6gAAAAA=",
        "AAAAAAAAAAAAAAAPY2xhaW1fZW1pc3Npb25zAAAAAAIAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAACw==",
        "AAAAAAAAAAAAAAAUY2xhaW1fZW1pc3Npb25zX2F1dGgAAAACAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAQAAA+gAAAPtAAAAAwAAABMAAAARAAAD6gAAAAA=",
        "AAAAAAAAAAAAAAANZ2V0X2VtaXNzaW9ucwAAAAAAAAEAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAOcHJvdG9jb2xfdG9rZW4AAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAATZ2V0X3RvdGFsX2RlcG9zaXRlZAAAAAABAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAACw==",
        "AAAAAAAAAAAAAAALZ2V0X2JhbGFuY2UAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAHZ2V0X2FweQAAAAABAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAABA==",
        "AAAAAAAAAAAAAAAWdXBkYXRlX2Vwb2NoX3ByaW5jaXBhbAAAAAAAAwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAVlcG9jaAAAAAAAAAYAAAAAAAAACXByaW5jaXBhbAAAAAAAAAsAAAAA",
        "AAAAAwAAAAAAAAAAAAAAC1JlcXVlc3RUeXBlAAAAAAoAAAAAAAAABlN1cHBseQAAAAAAAAAAAAAAAAAIV2l0aGRyYXcAAAABAAAAAAAAABBTdXBwbHlDb2xsYXRlcmFsAAAAAgAAAAAAAAASV2l0aGRyYXdDb2xsYXRlcmFsAAAAAAADAAAAAAAAAAZCb3Jyb3cAAAAAAAQAAAAAAAAABVJlcGF5AAAAAAAABQAAAAAAAAAaRmlsbFVzZXJMaXF1aWRhdGlvbkF1Y3Rpb24AAAAAAAYAAAAAAAAAEkZpbGxCYWREZWJ0QXVjdGlvbgAAAAAABwAAAAAAAAATRmlsbEludGVyZXN0QXVjdGlvbgAAAAAIAAAAAAAAABhEZWxldGVMaXF1aWRhdGlvbkF1Y3Rpb24AAAAJ",
        "AAAAAQAAAAAAAAAAAAAAB1JlcXVlc3QAAAAAAwAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAADHJlcXVlc3RfdHlwZQAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAE0Fzc2V0RXBvY2hQcmluY2lwYWwAAAAABQAAAAAAAAARZGVwb3NpdHNfaW5fZXBvY2gAAAAAAAALAAAAAAAAAAVlcG9jaAAAAAAAAAYAAAAAAAAADGxhc3RfdXBkYXRlZAAAAAYAAAAAAAAACXByaW5jaXBhbAAAAAAAAAsAAAAAAAAAC3dpdGhkcmF3YWxzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAEAAAAAAAAACmJzdG9wX3JhdGUAAAAAAAQ=",
        "AAAAAAAAAAAAAAAEaW5pdAAAAAEAAAAAAAAADWluaXRpYWxfYXNzZXQAAAAAAAATAAAAAQAAA+0AAAAA",
        "AAAAAAAAAAAAAAAGc3VibWl0AAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAIX3NwZW5kZXIAAAATAAAAAAAAAANfdG8AAAAAEwAAAAAAAAAIcmVxdWVzdHMAAAPqAAAH0AAAAAdSZXF1ZXN0AAAAAAEAAAfQAAAACVBvc2l0aW9ucwAAAA==",
        "AAAAAAAAAAAAAAAVc3VibWl0X3dpdGhfYWxsb3dhbmNlAAAAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAACF9zcGVuZGVyAAAAEwAAAAAAAAAHX3NlbmRlcgAAAAATAAAAAAAAAAlfcmVxdWVzdHMAAAAAAAPqAAAH0AAAAAdSZXF1ZXN0AAAAAAEAAAfQAAAACVBvc2l0aW9ucwAAAA==",
        "AAAAAAAAAAAAAAANZ2V0X3Bvc2l0aW9ucwAAAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAB9AAAAAJUG9zaXRpb25zAAAA",
        "AAAAAAAAAAAAAAALZ2V0X3Jlc2VydmUAAAAAAQAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAQAAB9AAAAAHUmVzZXJ2ZQA=",
        "AAAAAAAAAAAAAAAQZ2V0X3Jlc2VydmVfbGlzdAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAKZ2V0X2NvbmZpZwAAAAAAAAAAAAEAAAfQAAAAClBvb2xDb25maWcAAA==",
        "AAAAAAAAAAAAAAAFY2xhaW0AAAAAAAADAAAAAAAAAAVfdXNlcgAAAAAAABMAAAAAAAAACl90b2tlbl9pZHMAAAAAA+oAAAAEAAAAAAAAAANfdG8AAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAANdXBkYXRlX2JfcmF0ZQAAAAAAAAIAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAIbmV3X3JhdGUAAAALAAAAAA==",
        "AAAAAAAAAAAAAAAJYWRkX3lpZWxkAAAAAAAAAgAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAx5aWVsZF9hbW91bnQAAAALAAAAAA==",
        "AAAAAAAAAAAAAAASZ2V0X3VzZXJfZW1pc3Npb25zAAAAAAACAAAAAAAAAAVfdXNlcgAAAAAAABMAAAAAAAAAEV9yZXNlcnZlX3Rva2VuX2lkAAAAAAAABAAAAAEAAAPoAAAH0AAAABBVc2VyRW1pc3Npb25EYXRh",
        "AAAAAgAAAAAAAAAAAAAAEFN1cHBvcnRlZEFkYXB0ZXIAAAACAAAAAAAAAAAAAAAMQmxlbmRDYXBpdGFsAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==",
        "AAAAAgAAAAAAAAAAAAAAElN1cHBvcnRlZFlpZWxkVHlwZQAAAAAAAwAAAAAAAAAAAAAAB0xlbmRpbmcAAAAAAAAAAAAAAAAJTGlxdWlkaXR5AAAAAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==" ]),
      options
    )
  }
  public readonly fromJSON = {
    deposit: this.txFromJSON<i128>,
        deposit_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        withdraw: this.txFromJSON<i128>,
        withdraw_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        get_yield: this.txFromJSON<i128>,
        claim_yield: this.txFromJSON<i128>,
        claim_yield_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        claim_emissions: this.txFromJSON<i128>,
        claim_emissions_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        get_emissions: this.txFromJSON<i128>,
        protocol_token: this.txFromJSON<string>,
        get_total_deposited: this.txFromJSON<i128>,
        get_balance: this.txFromJSON<i128>,
        get_apy: this.txFromJSON<u32>,
        update_epoch_principal: this.txFromJSON<null>,
        init: this.txFromJSON<void>,
        submit: this.txFromJSON<Positions>,
        submit_with_allowance: this.txFromJSON<Positions>,
        get_positions: this.txFromJSON<Positions>,
        get_reserve: this.txFromJSON<Reserve>,
        get_reserve_list: this.txFromJSON<Array<string>>,
        get_config: this.txFromJSON<PoolConfig>,
        claim: this.txFromJSON<i128>,
        update_b_rate: this.txFromJSON<null>,
        add_yield: this.txFromJSON<null>,
        get_user_emissions: this.txFromJSON<Option<UserEmissionData>>
  }
}