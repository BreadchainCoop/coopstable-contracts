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




export type DataKey = {tag: "Admin", values: void} | {tag: "Manager", values: void};

/**
 * Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
 * CUSDManagerError error reporting. CUSDManager specific errors start at 100
 */
export const CUSDManagerError = {
  1: {message:"InternalError"},
  3: {message:"AlreadyInitializedError"},
  4: {message:"UnauthorizedError"},
  8: {message:"NegativeAmountError"},
  10: {message:"BalanceError"},
  12: {message:"OverflowError"}
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

export const AccessControlError = {
  1: {message:"InternalError"},
  3: {message:"AlreadyInitializedError"},
  4: {message:"UnauthorizedError"},
  8: {message:"NegativeAmountError"},
  10: {message:"BalanceError"},
  12: {message:"OverflowError"},
  1300: {message:"OnlyRoleAdmin"},
  1301: {message:"UnAuhtorizedRole"}
}

export interface Client {
  /**
   * Construct and simulate a set_default_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_default_admin: ({caller, new_admin}: {caller: string, new_admin: string}, options?: {
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
   * Construct and simulate a set_cusd_manager_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_cusd_manager_admin: ({caller, new_admin}: {caller: string, new_admin: string}, options?: {
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
   * Construct and simulate a set_cusd_issuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_cusd_issuer: ({caller, new_issuer}: {caller: string, new_issuer: string}, options?: {
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
   * Construct and simulate a get_cusd_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_cusd_id: (options?: {
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
   * Construct and simulate a issue_cusd transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_cusd: ({caller, to, amount}: {caller: string, to: string, amount: i128}, options?: {
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
   * Construct and simulate a set_yield_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_controller: ({caller, new_controller}: {caller: string, new_controller: string}, options?: {
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
   * Construct and simulate a burn_cusd transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  burn_cusd: ({caller, from, amount}: {caller: string, from: string, amount: i128}, options?: {
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
        {cusd_id, owner, admin}: {cusd_id: string, owner: string, admin: string},
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
    return ContractClient.deploy({cusd_id, owner, admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAMAAAAAAAAAB2N1c2RfaWQAAAAAEwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAARc2V0X2RlZmF1bHRfYWRtaW4AAAAAAAACAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAACW5ld19hZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAWc2V0X2N1c2RfbWFuYWdlcl9hZG1pbgAAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAPc2V0X2N1c2RfaXNzdWVyAAAAAAIAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAKbmV3X2lzc3VlcgAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAALZ2V0X2N1c2RfaWQAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAKaXNzdWVfY3VzZAAAAAAAAwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAA==",
        "AAAAAAAAAAAAAAAUc2V0X3lpZWxkX2NvbnRyb2xsZXIAAAACAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAADm5ld19jb250cm9sbGVyAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAJYnVybl9jdXNkAAAAAAAAAwAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAARmcm9tAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAA==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAHTWFuYWdlcgA=",
        "AAAABAAAAK1FcnJvciBjb2RlcyBmb3IgdGhlIGN1c2RfbWFuYWdlciBjb250cmFjdC4gQ29tbW9uIGVycm9ycyBhcmUgY29kZXMgdGhhdCBtYXRjaCB1cCB3aXRoIHRoZSBidWlsdC1pbgpDVVNETWFuYWdlckVycm9yIGVycm9yIHJlcG9ydGluZy4gQ1VTRE1hbmFnZXIgc3BlY2lmaWMgZXJyb3JzIHN0YXJ0IGF0IDEwMAAAAAAAAAAAAAAQQ1VTRE1hbmFnZXJFcnJvcgAAAAYAAAAAAAAADUludGVybmFsRXJyb3IAAAAAAAABAAAAAAAAABdBbHJlYWR5SW5pdGlhbGl6ZWRFcnJvcgAAAAADAAAAAAAAABFVbmF1dGhvcml6ZWRFcnJvcgAAAAAAAAQAAAAAAAAAE05lZ2F0aXZlQW1vdW50RXJyb3IAAAAACAAAAAAAAAAMQmFsYW5jZUVycm9yAAAACgAAAAAAAAANT3ZlcmZsb3dFcnJvcgAAAAAAAAw=",
        "AAAAAQAAAAAAAAAAAAAACFJvbGVEYXRhAAAAAgAAAAAAAAAKYWRtaW5fcm9sZQAAAAAAEQAAAAAAAAAHbWVtYmVycwAAAAPsAAAAEwAAAAE=",
        "AAAAAQAAADFBIHN0b3JhZ2Ugc3RydWN0dXJlIGZvciBhbGwgcm9sZXMgaW4gdGhlIGNvbnRyYWN0AAAAAAAAAAAAAAhSb2xlc01hcAAAAAEAAAAAAAAABXJvbGVzAAAAAAAD7AAAABEAAAfQAAAACFJvbGVEYXRh",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAACAAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAEAAAAAAAAAF0FscmVhZHlJbml0aWFsaXplZEVycm9yAAAAAAMAAAAAAAAAEVVuYXV0aG9yaXplZEVycm9yAAAAAAAABAAAAAAAAAATTmVnYXRpdmVBbW91bnRFcnJvcgAAAAAIAAAAAAAAAAxCYWxhbmNlRXJyb3IAAAAKAAAAAAAAAA1PdmVyZmxvd0Vycm9yAAAAAAAADAAAAAAAAAANT25seVJvbGVBZG1pbgAAAAAABRQAAAAAAAAAEFVuQXVodG9yaXplZFJvbGUAAAUV" ]),
      options
    )
  }
  public readonly fromJSON = {
    set_default_admin: this.txFromJSON<null>,
        set_cusd_manager_admin: this.txFromJSON<null>,
        set_cusd_issuer: this.txFromJSON<null>,
        get_cusd_id: this.txFromJSON<string>,
        issue_cusd: this.txFromJSON<null>,
        set_yield_controller: this.txFromJSON<null>,
        burn_cusd: this.txFromJSON<null>
  }
}