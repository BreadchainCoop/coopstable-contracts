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





export interface RoleData {
  admin_role: string;
  members: Map<string, boolean>;
}


/**
 * A storage structure for all roles in the contract
 */
export interface RolesMap {
  roles: Map<string, RoleData>;
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
   * Construct and simulate a deposit_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_collateral: ({protocol, user, asset, amount}: {protocol: string, user: string, asset: string, amount: i128}, options?: {
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
  withdraw_collateral: ({protocol, user, asset, amount}: {protocol: string, user: string, asset: string, amount: i128}, options?: {
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
  get_yield: (options?: {
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
  claim_yield: (options?: {
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
   * Construct and simulate a set_yield_distributor transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_distributor: ({yield_distributor}: {yield_distributor: string}, options?: {
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
   * Construct and simulate a set_adapter_registry transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_adapter_registry: ({adapter_registry}: {adapter_registry: string}, options?: {
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
   * Construct and simulate a set_cusd_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_cusd_manager: ({cusd_manager}: {cusd_manager: string}, options?: {
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
   * Construct and simulate a get_yield_distributor transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_yield_distributor: (options?: {
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
   * Construct and simulate a get_adapter_registry transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapter_registry: (options?: {
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
   * Construct and simulate a get_cusd_manager transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_cusd_manager: (options?: {
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
   * Construct and simulate a set_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_admin: ({caller, new_admin}: {caller: string, new_admin: string}, options?: {
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
        {yield_distributor, adapter_registry, cusd_manager, admin, owner}: {yield_distributor: string, adapter_registry: string, cusd_manager: string, admin: string, owner: string},
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
    return ContractClient.deploy({yield_distributor, adapter_registry, cusd_manager, admin, owner}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAUAAAAAAAAAEXlpZWxkX2Rpc3RyaWJ1dG9yAAAAAAAAEwAAAAAAAAAQYWRhcHRlcl9yZWdpc3RyeQAAABMAAAAAAAAADGN1c2RfbWFuYWdlcgAAABMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAASZGVwb3NpdF9jb2xsYXRlcmFsAAAAAAAEAAAAAAAAAAhwcm90b2NvbAAAABEAAAAAAAAABHVzZXIAAAATAAAAAAAAAAVhc3NldAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAAL",
        "AAAAAAAAAAAAAAATd2l0aGRyYXdfY29sbGF0ZXJhbAAAAAAEAAAAAAAAAAhwcm90b2NvbAAAABEAAAAAAAAABHVzZXIAAAATAAAAAAAAAAVhc3NldAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAJZ2V0X3lpZWxkAAAAAAAAAAAAAAEAAAAL",
        "AAAAAAAAAAAAAAALY2xhaW1feWllbGQAAAAAAAAAAAEAAAAL",
        "AAAAAAAAAAAAAAAVc2V0X3lpZWxkX2Rpc3RyaWJ1dG9yAAAAAAAAAQAAAAAAAAAReWllbGRfZGlzdHJpYnV0b3IAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAUc2V0X2FkYXB0ZXJfcmVnaXN0cnkAAAABAAAAAAAAABBhZGFwdGVyX3JlZ2lzdHJ5AAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAQc2V0X2N1c2RfbWFuYWdlcgAAAAEAAAAAAAAADGN1c2RfbWFuYWdlcgAAABMAAAAA",
        "AAAAAAAAAAAAAAAVZ2V0X3lpZWxkX2Rpc3RyaWJ1dG9yAAAAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAUZ2V0X2FkYXB0ZXJfcmVnaXN0cnkAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAQZ2V0X2N1c2RfbWFuYWdlcgAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAJc2V0X2FkbWluAAAAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAQAAAAAAAAAAAAAACFJvbGVEYXRhAAAAAgAAAAAAAAAKYWRtaW5fcm9sZQAAAAAAEQAAAAAAAAAHbWVtYmVycwAAAAPsAAAAEwAAAAE=",
        "AAAAAQAAADFBIHN0b3JhZ2Ugc3RydWN0dXJlIGZvciBhbGwgcm9sZXMgaW4gdGhlIGNvbnRyYWN0AAAAAAAAAAAAAAhSb2xlc01hcAAAAAEAAAAAAAAABXJvbGVzAAAAAAAD7AAAABEAAAfQAAAACFJvbGVEYXRh",
        "AAAAAgAAAAAAAAAAAAAAEFN1cHBvcnRlZEFkYXB0ZXIAAAACAAAAAAAAAAAAAAAMQmxlbmRDYXBpdGFsAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==",
        "AAAAAgAAAAAAAAAAAAAAElN1cHBvcnRlZFlpZWxkVHlwZQAAAAAAAwAAAAAAAAAAAAAAB0xlbmRpbmcAAAAAAAAAAAAAAAAJTGlxdWlkaXR5AAAAAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==",
        "AAAABAAAAAAAAAAAAAAADEFkYXB0ZXJFcnJvcgAAAAMAAAAAAAAAE0luc3VmZmljaWVudEJhbGFuY2UAAAAAAQAAAAAAAAAWTGVuZGluZ09wZXJhdGlvbkZhaWxlZAAAAAAAAgAAAAAAAAAMVW5hdXRob3JpemVkAAAAAw==" ]),
      options
    )
  }
  public readonly fromJSON = {
    deposit_collateral: this.txFromJSON<i128>,
        withdraw_collateral: this.txFromJSON<i128>,
        get_yield: this.txFromJSON<i128>,
        claim_yield: this.txFromJSON<i128>,
        set_yield_distributor: this.txFromJSON<null>,
        set_adapter_registry: this.txFromJSON<null>,
        set_cusd_manager: this.txFromJSON<null>,
        get_yield_distributor: this.txFromJSON<string>,
        get_adapter_registry: this.txFromJSON<string>,
        get_cusd_manager: this.txFromJSON<string>,
        set_admin: this.txFromJSON<null>
  }
}