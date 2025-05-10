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

export type SupportedAdapter = {tag: "BlendCapital", values: void} | {tag: "Custom", values: readonly [string]};

export type SupportedYieldType = {tag: "Lending", values: void} | {tag: "Liquidity", values: void} | {tag: "Custom", values: readonly [string]};

export const Errors = {
  1: {message:"InsufficientBalance"},

  2: {message:"LendingOperationFailed"},

  3: {message:"Unauthorized"}
}

export interface Client {
  /**
   * Construct and simulate a create_request transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  create_request: ({request_type, asset, amount}: {request_type: RequestType, asset: string, amount: i128}, options?: {
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
  }) => Promise<AssembledTransaction<Request>>

  /**
   * Construct and simulate a supply_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  supply_collateral: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: {
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
   * Construct and simulate a withdraw_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_collateral: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: {
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
   * Construct and simulate a get_reserve_token_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_reserve_token_id: ({asset}: {asset: string}, options?: {
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
  }) => Promise<AssembledTransaction<Option<u32>>>

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
   * Construct and simulate a get_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_yield: ({user, asset}: {user: string, asset: string}, options?: {
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
  claim_yield: ({user, asset}: {user: string, asset: string}, options?: {
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
  get_positions: ({_user}: {_user: string}, options?: {
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
  update_b_rate: ({_asset, _new_rate}: {_asset: string, _new_rate: i128}, options?: {
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

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {lending_adapter_controller_id, lending_pool_id}: {lending_adapter_controller_id: string, lending_pool_id: string},
    /** Options for initalizing a Client as well as for calling a method, with extras specific to deploying. */
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
    return ContractClient.deploy({lending_adapter_controller_id, lending_pool_id}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAAOY3JlYXRlX3JlcXVlc3QAAAAAAAMAAAAAAAAADHJlcXVlc3RfdHlwZQAAB9AAAAALUmVxdWVzdFR5cGUAAAAAAAAAAAVhc3NldAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAfQAAAAB1JlcXVlc3QA",
        "AAAAAAAAAAAAAAARc3VwcGx5X2NvbGxhdGVyYWwAAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAATd2l0aGRyYXdfY29sbGF0ZXJhbAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAALZ2V0X2JhbGFuY2UAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAUZ2V0X3Jlc2VydmVfdG9rZW5faWQAAAABAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAD6AAAAAQ=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAAHWxlbmRpbmdfYWRhcHRlcl9jb250cm9sbGVyX2lkAAAAAAAAEwAAAAAAAAAPbGVuZGluZ19wb29sX2lkAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAAJZ2V0X3lpZWxkAAAAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAALY2xhaW1feWllbGQAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAwAAAAAAAAAAAAAAC1JlcXVlc3RUeXBlAAAAAAoAAAAAAAAABlN1cHBseQAAAAAAAAAAAAAAAAAIV2l0aGRyYXcAAAABAAAAAAAAABBTdXBwbHlDb2xsYXRlcmFsAAAAAgAAAAAAAAASV2l0aGRyYXdDb2xsYXRlcmFsAAAAAAADAAAAAAAAAAZCb3Jyb3cAAAAAAAQAAAAAAAAABVJlcGF5AAAAAAAABQAAAAAAAAAaRmlsbFVzZXJMaXF1aWRhdGlvbkF1Y3Rpb24AAAAAAAYAAAAAAAAAEkZpbGxCYWREZWJ0QXVjdGlvbgAAAAAABwAAAAAAAAATRmlsbEludGVyZXN0QXVjdGlvbgAAAAAIAAAAAAAAABhEZWxldGVMaXF1aWRhdGlvbkF1Y3Rpb24AAAAJ",
        "AAAAAQAAAAAAAAAAAAAAB1JlcXVlc3QAAAAAAwAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAADHJlcXVlc3RfdHlwZQAAAAQ=",
        "AAAAAAAAAAAAAAAEaW5pdAAAAAEAAAAAAAAADWluaXRpYWxfYXNzZXQAAAAAAAATAAAAAQAAA+0AAAAA",
        "AAAAAAAAAAAAAAAVc3VibWl0X3dpdGhfYWxsb3dhbmNlAAAAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAACF9zcGVuZGVyAAAAEwAAAAAAAAAHX3NlbmRlcgAAAAATAAAAAAAAAAlfcmVxdWVzdHMAAAAAAAPqAAAH0AAAAAdSZXF1ZXN0AAAAAAEAAAfQAAAACVBvc2l0aW9ucwAAAA==",
        "AAAAAAAAAAAAAAANZ2V0X3Bvc2l0aW9ucwAAAAAAAAEAAAAAAAAABV91c2VyAAAAAAAAEwAAAAEAAAfQAAAACVBvc2l0aW9ucwAAAA==",
        "AAAAAAAAAAAAAAALZ2V0X3Jlc2VydmUAAAAAAQAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAQAAB9AAAAAHUmVzZXJ2ZQA=",
        "AAAAAAAAAAAAAAAQZ2V0X3Jlc2VydmVfbGlzdAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAFY2xhaW0AAAAAAAADAAAAAAAAAAVfdXNlcgAAAAAAABMAAAAAAAAACl90b2tlbl9pZHMAAAAAA+oAAAAEAAAAAAAAAANfdG8AAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAANdXBkYXRlX2JfcmF0ZQAAAAAAAAIAAAAAAAAABl9hc3NldAAAAAAAEwAAAAAAAAAJX25ld19yYXRlAAAAAAAACwAAAAA=",
        "AAAAAgAAAAAAAAAAAAAAEFN1cHBvcnRlZEFkYXB0ZXIAAAACAAAAAAAAAAAAAAAMQmxlbmRDYXBpdGFsAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==",
        "AAAAAgAAAAAAAAAAAAAAElN1cHBvcnRlZFlpZWxkVHlwZQAAAAAAAwAAAAAAAAAAAAAAB0xlbmRpbmcAAAAAAAAAAAAAAAAJTGlxdWlkaXR5AAAAAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==",
        "AAAABAAAAAAAAAAAAAAADEFkYXB0ZXJFcnJvcgAAAAMAAAAAAAAAE0luc3VmZmljaWVudEJhbGFuY2UAAAAAAQAAAAAAAAAWTGVuZGluZ09wZXJhdGlvbkZhaWxlZAAAAAAAAgAAAAAAAAAMVW5hdXRob3JpemVkAAAAAw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    create_request: this.txFromJSON<Request>,
        supply_collateral: this.txFromJSON<i128>,
        withdraw_collateral: this.txFromJSON<i128>,
        get_balance: this.txFromJSON<i128>,
        get_reserve_token_id: this.txFromJSON<Option<u32>>,
        deposit: this.txFromJSON<i128>,
        withdraw: this.txFromJSON<i128>,
        get_yield: this.txFromJSON<i128>,
        claim_yield: this.txFromJSON<i128>,
        init: this.txFromJSON<void>,
        submit_with_allowance: this.txFromJSON<Positions>,
        get_positions: this.txFromJSON<Positions>,
        get_reserve: this.txFromJSON<Reserve>,
        get_reserve_list: this.txFromJSON<Array<string>>,
        claim: this.txFromJSON<i128>,
        update_b_rate: this.txFromJSON<null>
  }
}