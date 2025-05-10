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





export interface DistributionConfig {
  distribution_period: u64;
  last_distribution: u64;
  treasury: string;
  treasury_share_bps: u32;
}


export interface Member {
  active: boolean;
  address: string;
  joined_at: u64;
}


export interface Distribution {
  member_amount: i128;
  member_count: u32;
  timestamp: u64;
  total_amount: i128;
  treasury_amount: i128;
}

export type DataKey = {tag: "Member", values: readonly [string]} | {tag: "Members", values: void} | {tag: "Distribution", values: readonly [u64]} | {tag: "Distributions", values: void};


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
   * Construct and simulate a set_yield_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_controller: ({caller, yield_controller}: {caller: string, yield_controller: string}, options?: {
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
   * Construct and simulate a add_member transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_member: ({caller, member}: {caller: string, member: string}, options?: {
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
   * Construct and simulate a remove_member transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_member: ({caller, member}: {caller: string, member: string}, options?: {
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
   * Construct and simulate a list_members transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  list_members: (options?: {
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
   * Construct and simulate a set_treasury transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_treasury: ({caller, treasury}: {caller: string, treasury: string}, options?: {
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
   * Construct and simulate a get_treasury transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_treasury: (options?: {
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
   * Construct and simulate a set_treasury_share transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_treasury_share: ({caller, share_bps}: {caller: string, share_bps: u32}, options?: {
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
   * Construct and simulate a get_treasury_share transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_treasury_share: (options?: {
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
   * Construct and simulate a set_distribution_period transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_distribution_period: ({caller, period}: {caller: string, period: u64}, options?: {
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
   * Construct and simulate a get_distribution_period transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_distribution_period: (options?: {
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
  }) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_next_distribution_time transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_next_distribution_time: (options?: {
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
  }) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a is_distribution_available transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_distribution_available: (options?: {
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
   * Construct and simulate a distribute_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_yield: ({caller, token, amount}: {caller: string, token: string, amount: i128}, options?: {
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
        {treasury, treasury_share_bps, yield_controller, distribution_period, owner, admin}: {treasury: string, treasury_share_bps: u32, yield_controller: string, distribution_period: u64, owner: string, admin: string},
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
    return ContractClient.deploy({treasury, treasury_share_bps, yield_controller, distribution_period, owner, admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAYAAAAAAAAACHRyZWFzdXJ5AAAAEwAAAAAAAAASdHJlYXN1cnlfc2hhcmVfYnBzAAAAAAAEAAAAAAAAABB5aWVsZF9jb250cm9sbGVyAAAAEwAAAAAAAAATZGlzdHJpYnV0aW9uX3BlcmlvZAAAAAAGAAAAAAAAAAVvd25lcgAAAAAAABMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAUc2V0X3lpZWxkX2NvbnRyb2xsZXIAAAACAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAAEHlpZWxkX2NvbnRyb2xsZXIAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAKYWRkX21lbWJlcgAAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAZtZW1iZXIAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAANcmVtb3ZlX21lbWJlcgAAAAAAAAIAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAGbWVtYmVyAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAMbGlzdF9tZW1iZXJzAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAMc2V0X3RyZWFzdXJ5AAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAh0cmVhc3VyeQAAABMAAAAA",
        "AAAAAAAAAAAAAAAMZ2V0X3RyZWFzdXJ5AAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAASc2V0X3RyZWFzdXJ5X3NoYXJlAAAAAAACAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAACXNoYXJlX2JwcwAAAAAAAAQAAAAA",
        "AAAAAAAAAAAAAAASZ2V0X3RyZWFzdXJ5X3NoYXJlAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAXc2V0X2Rpc3RyaWJ1dGlvbl9wZXJpb2QAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAZwZXJpb2QAAAAAAAYAAAAA",
        "AAAAAAAAAAAAAAAXZ2V0X2Rpc3RyaWJ1dGlvbl9wZXJpb2QAAAAAAAAAAAEAAAAG",
        "AAAAAAAAAAAAAAAaZ2V0X25leHRfZGlzdHJpYnV0aW9uX3RpbWUAAAAAAAAAAAABAAAABg==",
        "AAAAAAAAAAAAAAAZaXNfZGlzdHJpYnV0aW9uX2F2YWlsYWJsZQAAAAAAAAAAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAQZGlzdHJpYnV0ZV95aWVsZAAAAAMAAAAAAAAABmNhbGxlcgAAAAAAEwAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAJc2V0X2FkbWluAAAAAAAAAgAAAAAAAAAGY2FsbGVyAAAAAAATAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAA==",
        "AAAAAQAAAAAAAAAAAAAAEkRpc3RyaWJ1dGlvbkNvbmZpZwAAAAAABAAAAAAAAAATZGlzdHJpYnV0aW9uX3BlcmlvZAAAAAAGAAAAAAAAABFsYXN0X2Rpc3RyaWJ1dGlvbgAAAAAAAAYAAAAAAAAACHRyZWFzdXJ5AAAAEwAAAAAAAAASdHJlYXN1cnlfc2hhcmVfYnBzAAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAABk1lbWJlcgAAAAAAAwAAAAAAAAAGYWN0aXZlAAAAAAABAAAAAAAAAAdhZGRyZXNzAAAAABMAAAAAAAAACWpvaW5lZF9hdAAAAAAAAAY=",
        "AAAAAQAAAAAAAAAAAAAADERpc3RyaWJ1dGlvbgAAAAUAAAAAAAAADW1lbWJlcl9hbW91bnQAAAAAAAALAAAAAAAAAAxtZW1iZXJfY291bnQAAAAEAAAAAAAAAAl0aW1lc3RhbXAAAAAAAAAGAAAAAAAAAAx0b3RhbF9hbW91bnQAAAALAAAAAAAAAA90cmVhc3VyeV9hbW91bnQAAAAACw==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABAAAAAEAAAAAAAAABk1lbWJlcgAAAAAAAQAAABMAAAAAAAAAAAAAAAdNZW1iZXJzAAAAAAEAAAAAAAAADERpc3RyaWJ1dGlvbgAAAAEAAAAGAAAAAAAAAAAAAAANRGlzdHJpYnV0aW9ucwAAAA==",
        "AAAAAQAAAAAAAAAAAAAACFJvbGVEYXRhAAAAAgAAAAAAAAAKYWRtaW5fcm9sZQAAAAAAEQAAAAAAAAAHbWVtYmVycwAAAAPsAAAAEwAAAAE=",
        "AAAAAQAAADFBIHN0b3JhZ2Ugc3RydWN0dXJlIGZvciBhbGwgcm9sZXMgaW4gdGhlIGNvbnRyYWN0AAAAAAAAAAAAAAhSb2xlc01hcAAAAAEAAAAAAAAABXJvbGVzAAAAAAAD7AAAABEAAAfQAAAACFJvbGVEYXRh" ]),
      options
    )
  }
  public readonly fromJSON = {
    set_yield_controller: this.txFromJSON<null>,
        add_member: this.txFromJSON<null>,
        remove_member: this.txFromJSON<null>,
        list_members: this.txFromJSON<Array<string>>,
        set_treasury: this.txFromJSON<null>,
        get_treasury: this.txFromJSON<string>,
        set_treasury_share: this.txFromJSON<null>,
        get_treasury_share: this.txFromJSON<u32>,
        set_distribution_period: this.txFromJSON<null>,
        get_distribution_period: this.txFromJSON<u64>,
        get_next_distribution_time: this.txFromJSON<u64>,
        is_distribution_available: this.txFromJSON<boolean>,
        distribute_yield: this.txFromJSON<boolean>,
        set_admin: this.txFromJSON<null>
  }
}