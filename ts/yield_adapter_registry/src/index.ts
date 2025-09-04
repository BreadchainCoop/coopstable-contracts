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





export interface YieldAdapterRegistryMap {
  registry_map: Map<string, string>;
  supported_assets: Map<string, Map<string, boolean>>;
  yield_type: string;
}

export type DataKey = {tag: "Owner", values: void} | {tag: "Admin", values: void};

/**
 * Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
 * YieldAdapterRegistry error reporting. YieldAdapterRegistry specific errors start at 400
 */
export const YieldAdapterRegistryError = {
  1: {message:"InternalError"},
  3: {message:"AlreadyInitializedError"},
  4: {message:"UnauthorizedError"},
  8: {message:"NegativeAmountError"},
  10: {message:"BalanceError"},
  12: {message:"OverflowError"},
  1100: {message:"InvalidYieldAdapter"}
}

export interface Client {
  /**
   * Construct and simulate a set_yield_adapter_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_adapter_admin: ({new_admin}: {new_admin: string}, options?: {
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
   * Construct and simulate a register_adapter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  register_adapter: ({yield_type, protocol, adapter_address}: {yield_type: string, protocol: string, adapter_address: string}, options?: {
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
   * Construct and simulate a remove_adapter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_adapter: ({yield_type, protocol}: {yield_type: string, protocol: string}, options?: {
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
   * Construct and simulate a get_adapter transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapter: ({yield_type, protocol}: {yield_type: string, protocol: string}, options?: {
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
   * Construct and simulate a add_support_for_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_support_for_asset: ({yield_type, protocol, asset_address}: {yield_type: string, protocol: string, asset_address: string}, options?: {
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
   * Construct and simulate a remove_support_for_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_support_for_asset: ({yield_type, protocol, asset_address}: {yield_type: string, protocol: string, asset_address: string}, options?: {
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
   * Construct and simulate a is_supported_asset transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_supported_asset: ({yield_type, protocol, asset_address}: {yield_type: string, protocol: string, asset_address: string}, options?: {
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
  }) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_adapters transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapters: ({yield_type}: {yield_type: string}, options?: {
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
   * Construct and simulate a get_adapters_with_assets transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_adapters_with_assets: ({yield_type}: {yield_type: string}, options?: {
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
  }) => Promise<AssembledTransaction<Array<readonly [string, Array<string>]>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, owner}: {admin: string, owner: string},
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
    return ContractClient.deploy({admin, owner}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAXc2V0X3lpZWxkX2FkYXB0ZXJfYWRtaW4AAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAQcmVnaXN0ZXJfYWRhcHRlcgAAAAMAAAAAAAAACnlpZWxkX3R5cGUAAAAAABEAAAAAAAAACHByb3RvY29sAAAAEQAAAAAAAAAPYWRhcHRlcl9hZGRyZXNzAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAOcmVtb3ZlX2FkYXB0ZXIAAAAAAAIAAAAAAAAACnlpZWxkX3R5cGUAAAAAABEAAAAAAAAACHByb3RvY29sAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAALZ2V0X2FkYXB0ZXIAAAAAAgAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAAAAAAIcHJvdG9jb2wAAAARAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAVYWRkX3N1cHBvcnRfZm9yX2Fzc2V0AAAAAAAAAwAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAAAAAAIcHJvdG9jb2wAAAARAAAAAAAAAA1hc3NldF9hZGRyZXNzAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAYcmVtb3ZlX3N1cHBvcnRfZm9yX2Fzc2V0AAAAAwAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAAAAAAIcHJvdG9jb2wAAAARAAAAAAAAAA1hc3NldF9hZGRyZXNzAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAASaXNfc3VwcG9ydGVkX2Fzc2V0AAAAAAADAAAAAAAAAAp5aWVsZF90eXBlAAAAAAARAAAAAAAAAAhwcm90b2NvbAAAABEAAAAAAAAADWFzc2V0X2FkZHJlc3MAAAAAAAATAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAMZ2V0X2FkYXB0ZXJzAAAAAQAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAYZ2V0X2FkYXB0ZXJzX3dpdGhfYXNzZXRzAAAAAQAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAEAAAPqAAAD7QAAAAIAAAATAAAD6gAAABM=",
        "AAAAAQAAAAAAAAAAAAAAF1lpZWxkQWRhcHRlclJlZ2lzdHJ5TWFwAAAAAAMAAAAAAAAADHJlZ2lzdHJ5X21hcAAAA+wAAAARAAAAEwAAAAAAAAAQc3VwcG9ydGVkX2Fzc2V0cwAAA+wAAAARAAAD7AAAABMAAAABAAAAAAAAAAp5aWVsZF90eXBlAAAAAAAR",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAAAAAAAAAAABU93bmVyAAAAAAAAAAAAAAAAAAAFQWRtaW4AAAA=",
        "AAAABAAAALpFcnJvciBjb2RlcyBmb3IgdGhlIGN1c2RfbWFuYWdlciBjb250cmFjdC4gQ29tbW9uIGVycm9ycyBhcmUgY29kZXMgdGhhdCBtYXRjaCB1cCB3aXRoIHRoZSBidWlsdC1pbgpZaWVsZEFkYXB0ZXJSZWdpc3RyeSBlcnJvciByZXBvcnRpbmcuIFlpZWxkQWRhcHRlclJlZ2lzdHJ5IHNwZWNpZmljIGVycm9ycyBzdGFydCBhdCA0MDAAAAAAAAAAAAAZWWllbGRBZGFwdGVyUmVnaXN0cnlFcnJvcgAAAAAAAAcAAAAAAAAADUludGVybmFsRXJyb3IAAAAAAAABAAAAAAAAABdBbHJlYWR5SW5pdGlhbGl6ZWRFcnJvcgAAAAADAAAAAAAAABFVbmF1dGhvcml6ZWRFcnJvcgAAAAAAAAQAAAAAAAAAE05lZ2F0aXZlQW1vdW50RXJyb3IAAAAACAAAAAAAAAAMQmFsYW5jZUVycm9yAAAACgAAAAAAAAANT3ZlcmZsb3dFcnJvcgAAAAAAAAwAAAAAAAAAE0ludmFsaWRZaWVsZEFkYXB0ZXIAAAAETA==" ]),
      options
    )
  }
  public readonly fromJSON = {
    set_yield_adapter_admin: this.txFromJSON<null>,
        register_adapter: this.txFromJSON<null>,
        remove_adapter: this.txFromJSON<null>,
        get_adapter: this.txFromJSON<string>,
        add_support_for_asset: this.txFromJSON<null>,
        remove_support_for_asset: this.txFromJSON<null>,
        is_supported_asset: this.txFromJSON<boolean>,
        get_adapters: this.txFromJSON<Array<string>>,
        get_adapters_with_assets: this.txFromJSON<Array<readonly [string, Array<string>]>>
  }
}