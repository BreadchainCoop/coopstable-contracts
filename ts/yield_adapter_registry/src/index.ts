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

export const Errors = {

}

export interface Client {
  /**
   * Construct and simulate a set_yield_adapter_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_adapter_admin: ({caller, new_admin}: {caller: string, new_admin: string}, options?: {
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
  register_adapter: ({caller, yield_type, protocol, adapter_address}: {caller: string, yield_type: string, protocol: string, adapter_address: string}, options?: {
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
  remove_adapter: ({caller, yield_type, protocol}: {caller: string, yield_type: string, protocol: string}, options?: {
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
  add_support_for_asset: ({caller, yield_type, protocol, asset_address}: {caller: string, yield_type: string, protocol: string, asset_address: string}, options?: {
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
  remove_support_for_asset: ({caller, yield_type, protocol, asset_address}: {caller: string, yield_type: string, protocol: string, asset_address: string}, options?: {
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
        {admin}: {admin: string},
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
    return ContractClient.deploy({admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAXc2V0X3lpZWxkX2FkYXB0ZXJfYWRtaW4AAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAQcmVnaXN0ZXJfYWRhcHRlcgAAAAQAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAAAAAAIcHJvdG9jb2wAAAARAAAAAAAAAA9hZGFwdGVyX2FkZHJlc3MAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAOcmVtb3ZlX2FkYXB0ZXIAAAAAAAMAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAAAAAAIcHJvdG9jb2wAAAARAAAAAA==",
        "AAAAAAAAAAAAAAALZ2V0X2FkYXB0ZXIAAAAAAgAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAAAAAAIcHJvdG9jb2wAAAARAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAVYWRkX3N1cHBvcnRfZm9yX2Fzc2V0AAAAAAAABAAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAp5aWVsZF90eXBlAAAAAAARAAAAAAAAAAhwcm90b2NvbAAAABEAAAAAAAAADWFzc2V0X2FkZHJlc3MAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAYcmVtb3ZlX3N1cHBvcnRfZm9yX2Fzc2V0AAAABAAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAp5aWVsZF90eXBlAAAAAAARAAAAAAAAAAhwcm90b2NvbAAAABEAAAAAAAAADWFzc2V0X2FkZHJlc3MAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAASaXNfc3VwcG9ydGVkX2Fzc2V0AAAAAAADAAAAAAAAAAp5aWVsZF90eXBlAAAAAAARAAAAAAAAAAhwcm90b2NvbAAAABEAAAAAAAAADWFzc2V0X2FkZHJlc3MAAAAAAAATAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAMZ2V0X2FkYXB0ZXJzAAAAAQAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAYZ2V0X2FkYXB0ZXJzX3dpdGhfYXNzZXRzAAAAAQAAAAAAAAAKeWllbGRfdHlwZQAAAAAAEQAAAAEAAAPqAAAD7QAAAAIAAAATAAAD6gAAABM=",
        "AAAAAQAAAAAAAAAAAAAAF1lpZWxkQWRhcHRlclJlZ2lzdHJ5TWFwAAAAAAMAAAAAAAAADHJlZ2lzdHJ5X21hcAAAA+wAAAARAAAAEwAAAAAAAAAQc3VwcG9ydGVkX2Fzc2V0cwAAA+wAAAARAAAD7AAAABMAAAABAAAAAAAAAAp5aWVsZF90eXBlAAAAAAAR",
        "AAAAAQAAAAAAAAAAAAAACFJvbGVEYXRhAAAAAgAAAAAAAAAKYWRtaW5fcm9sZQAAAAAAEQAAAAAAAAAHbWVtYmVycwAAAAPsAAAAEwAAAAE=",
        "AAAAAQAAADFBIHN0b3JhZ2Ugc3RydWN0dXJlIGZvciBhbGwgcm9sZXMgaW4gdGhlIGNvbnRyYWN0AAAAAAAAAAAAAAhSb2xlc01hcAAAAAEAAAAAAAAABXJvbGVzAAAAAAAD7AAAABEAAAfQAAAACFJvbGVEYXRh" ]),
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