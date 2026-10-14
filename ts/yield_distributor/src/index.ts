import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
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
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}





export interface Member {
  active: boolean;
  address: string;
  joined_at: u64;
}

export type DataKey = {tag: "Admin", values: void} | {tag: "Owner", values: void} | {tag: "YieldController", values: void} | {tag: "Treasury", values: void} | {tag: "Member", values: readonly [string]} | {tag: "Members", values: void} | {tag: "Distributions", values: void} | {tag: "Distribution", values: readonly [u64]} | {tag: "DistributionConfig", values: void} | {tag: "Epoch", values: readonly [u64]} | {tag: "EpochStartTimestamp", values: readonly [u64]} | {tag: "TotalDistributed", values: void};


export interface Distribution {
  distribution_end_timestamp: u64;
  distribution_member: i128;
  distribution_start_timestamp: u64;
  distribution_total: i128;
  distribution_treasury: i128;
  epoch: u64;
  is_processed: boolean;
  members: Array<string>;
}


export interface DistributionConfig {
  distribution_period: u64;
  treasury_share_bps: u32;
}

/**
 * Error codes for the cusd_manager contract. Common errors are codes that match up with the built-in
 * YieldDistributorError error reporting. YieldDistributor specific errors start at 400
 */
export const YieldDistributorError = {
  1: {message:"InternalError"},
  3: {message:"AlreadyInitializedError"},
  4: {message:"UnauthorizedError"},
  8: {message:"NegativeAmountError"},
  10: {message:"BalanceError"},
  12: {message:"OverflowError"},
  1200: {message:"MemberAlreadyExists"},
  1201: {message:"MemberDoesNotExist"}
}

export interface Client {
  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a add_member transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_member: ({member}: {member: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_treasury transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_treasury: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a list_members transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  list_members: (options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a set_treasury transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_treasury: ({treasury}: {treasury: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a advance_epoch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  advance_epoch: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a remove_member transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_member: ({member}: {member: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a distribute_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_yield: ({token, amount}: {token: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_current_epoch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_current_epoch: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a get_treasury_share transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_treasury_share: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a set_treasury_share transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_treasury_share: ({share_bps}: {share_bps: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_yield_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_yield_controller: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a set_yield_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_controller: ({yield_controller}: {yield_controller: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_distribution_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_distribution_info: (options?: MethodOptions) => Promise<AssembledTransaction<Distribution>>

  /**
   * Construct and simulate a get_total_distributed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_distributed: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_distribution_period transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_distribution_period: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a set_distribution_period transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_distribution_period: ({period}: {period: u64}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_distribution_history transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_distribution_history: (options?: MethodOptions) => Promise<AssembledTransaction<Array<Distribution>>>

  /**
   * Construct and simulate a is_distribution_available transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  is_distribution_available: (options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_next_distribution_time transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_next_distribution_time: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a time_before_next_distribution transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  time_before_next_distribution: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {treasury, treasury_share_bps, yield_controller, distribution_period, owner, admin}: {treasury: string, treasury_share_bps: u32, yield_controller: string, distribution_period: u64, owner: string, admin: string},
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
    return ContractClient.deploy({treasury, treasury_share_bps, yield_controller, distribution_period, owner, admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAABk1lbWJlcgAAAAAAAwAAAAAAAAAGYWN0aXZlAAAAAAABAAAAAAAAAAdhZGRyZXNzAAAAABMAAAAAAAAACWpvaW5lZF9hdAAAAAAAAAY=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAADAAAAAAAAAAAAAAABUFkbWluAAAAAAAAAAAAAAAAAAAFT3duZXIAAAAAAAAAAAAAAAAAAA9ZaWVsZENvbnRyb2xsZXIAAAAAAAAAAAAAAAAIVHJlYXN1cnkAAAABAAAAAAAAAAZNZW1iZXIAAAAAAAEAAAATAAAAAAAAAAAAAAAHTWVtYmVycwAAAAAAAAAAAAAAAA1EaXN0cmlidXRpb25zAAAAAAAAAQAAAAAAAAAMRGlzdHJpYnV0aW9uAAAAAQAAAAYAAAAAAAAAAAAAABJEaXN0cmlidXRpb25Db25maWcAAAAAAAEAAAAAAAAABUVwb2NoAAAAAAAAAQAAAAYAAAABAAAAAAAAABNFcG9jaFN0YXJ0VGltZXN0YW1wAAAAAAEAAAAGAAAAAAAAAAAAAAAQVG90YWxEaXN0cmlidXRlZA==",
        "AAAAAQAAAAAAAAAAAAAADERpc3RyaWJ1dGlvbgAAAAgAAAAAAAAAGmRpc3RyaWJ1dGlvbl9lbmRfdGltZXN0YW1wAAAAAAAGAAAAAAAAABNkaXN0cmlidXRpb25fbWVtYmVyAAAAAAsAAAAAAAAAHGRpc3RyaWJ1dGlvbl9zdGFydF90aW1lc3RhbXAAAAAGAAAAAAAAABJkaXN0cmlidXRpb25fdG90YWwAAAAAAAsAAAAAAAAAFWRpc3RyaWJ1dGlvbl90cmVhc3VyeQAAAAAAAAsAAAAAAAAABWVwb2NoAAAAAAAABgAAAAAAAAAMaXNfcHJvY2Vzc2VkAAAAAQAAAAAAAAAHbWVtYmVycwAAAAPqAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAAEkRpc3RyaWJ1dGlvbkNvbmZpZwAAAAAAAgAAAAAAAAATZGlzdHJpYnV0aW9uX3BlcmlvZAAAAAAGAAAAAAAAABJ0cmVhc3VyeV9zaGFyZV9icHMAAAAAAAQ=",
        "AAAABAAAALdFcnJvciBjb2RlcyBmb3IgdGhlIGN1c2RfbWFuYWdlciBjb250cmFjdC4gQ29tbW9uIGVycm9ycyBhcmUgY29kZXMgdGhhdCBtYXRjaCB1cCB3aXRoIHRoZSBidWlsdC1pbgpZaWVsZERpc3RyaWJ1dG9yRXJyb3IgZXJyb3IgcmVwb3J0aW5nLiBZaWVsZERpc3RyaWJ1dG9yIHNwZWNpZmljIGVycm9ycyBzdGFydCBhdCA0MDAAAAAAAAAAABVZaWVsZERpc3RyaWJ1dG9yRXJyb3IAAAAAAAAIAAAAAAAAAA1JbnRlcm5hbEVycm9yAAAAAAAAAQAAAAAAAAAXQWxyZWFkeUluaXRpYWxpemVkRXJyb3IAAAAAAwAAAAAAAAARVW5hdXRob3JpemVkRXJyb3IAAAAAAAAEAAAAAAAAABNOZWdhdGl2ZUFtb3VudEVycm9yAAAAAAgAAAAAAAAADEJhbGFuY2VFcnJvcgAAAAoAAAAAAAAADU92ZXJmbG93RXJyb3IAAAAAAAAMAAAAAAAAABNNZW1iZXJBbHJlYWR5RXhpc3RzAAAABLAAAAAAAAAAEk1lbWJlckRvZXNOb3RFeGlzdAAAAAAEsQ==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAAJc2V0X2FkbWluAAAAAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAKYWRkX21lbWJlcgAAAAAAAQAAAAAAAAAGbWVtYmVyAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAMZ2V0X3RyZWFzdXJ5AAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAMbGlzdF9tZW1iZXJzAAAAAAAAAAEAAAPqAAAAEw==",
        "AAAAAAAAAAAAAAAMc2V0X3RyZWFzdXJ5AAAAAQAAAAAAAAAIdHJlYXN1cnkAAAATAAAAAA==",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAYAAAAAAAAACHRyZWFzdXJ5AAAAEwAAAAAAAAASdHJlYXN1cnlfc2hhcmVfYnBzAAAAAAAEAAAAAAAAABB5aWVsZF9jb250cm9sbGVyAAAAEwAAAAAAAAATZGlzdHJpYnV0aW9uX3BlcmlvZAAAAAAGAAAAAAAAAAVvd25lcgAAAAAAABMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAANYWR2YW5jZV9lcG9jaAAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAANcmVtb3ZlX21lbWJlcgAAAAAAAAEAAAAAAAAABm1lbWJlcgAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAQZGlzdHJpYnV0ZV95aWVsZAAAAAIAAAAAAAAABXRva2VuAAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAARZ2V0X2N1cnJlbnRfZXBvY2gAAAAAAAAAAAAAAQAAAAY=",
        "AAAAAAAAAAAAAAASZ2V0X3RyZWFzdXJ5X3NoYXJlAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAASc2V0X3RyZWFzdXJ5X3NoYXJlAAAAAAABAAAAAAAAAAlzaGFyZV9icHMAAAAAAAAEAAAAAA==",
        "AAAAAAAAAAAAAAAUZ2V0X3lpZWxkX2NvbnRyb2xsZXIAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAUc2V0X3lpZWxkX2NvbnRyb2xsZXIAAAABAAAAAAAAABB5aWVsZF9jb250cm9sbGVyAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAVZ2V0X2Rpc3RyaWJ1dGlvbl9pbmZvAAAAAAAAAAAAAAEAAAfQAAAADERpc3RyaWJ1dGlvbg==",
        "AAAAAAAAAAAAAAAVZ2V0X3RvdGFsX2Rpc3RyaWJ1dGVkAAAAAAAAAAAAAAEAAAAL",
        "AAAAAAAAAAAAAAAXZ2V0X2Rpc3RyaWJ1dGlvbl9wZXJpb2QAAAAAAAAAAAEAAAAG",
        "AAAAAAAAAAAAAAAXc2V0X2Rpc3RyaWJ1dGlvbl9wZXJpb2QAAAAAAQAAAAAAAAAGcGVyaW9kAAAAAAAGAAAAAA==",
        "AAAAAAAAAAAAAAAYZ2V0X2Rpc3RyaWJ1dGlvbl9oaXN0b3J5AAAAAAAAAAEAAAPqAAAH0AAAAAxEaXN0cmlidXRpb24=",
        "AAAAAAAAAAAAAAAZaXNfZGlzdHJpYnV0aW9uX2F2YWlsYWJsZQAAAAAAAAAAAAABAAAAAQ==",
        "AAAAAAAAAAAAAAAaZ2V0X25leHRfZGlzdHJpYnV0aW9uX3RpbWUAAAAAAAAAAAABAAAABg==",
        "AAAAAAAAAAAAAAAddGltZV9iZWZvcmVfbmV4dF9kaXN0cmlidXRpb24AAAAAAAAAAAAAAQAAAAY=" ]),
      options
    )
  }
  public readonly fromJSON = {
    upgrade: this.txFromJSON<null>,
        set_admin: this.txFromJSON<null>,
        add_member: this.txFromJSON<null>,
        get_treasury: this.txFromJSON<string>,
        list_members: this.txFromJSON<Array<string>>,
        set_treasury: this.txFromJSON<null>,
        advance_epoch: this.txFromJSON<null>,
        remove_member: this.txFromJSON<null>,
        distribute_yield: this.txFromJSON<i128>,
        get_current_epoch: this.txFromJSON<u64>,
        get_treasury_share: this.txFromJSON<u32>,
        set_treasury_share: this.txFromJSON<null>,
        get_yield_controller: this.txFromJSON<string>,
        set_yield_controller: this.txFromJSON<null>,
        get_distribution_info: this.txFromJSON<Distribution>,
        get_total_distributed: this.txFromJSON<i128>,
        get_distribution_period: this.txFromJSON<u64>,
        set_distribution_period: this.txFromJSON<null>,
        get_distribution_history: this.txFromJSON<Array<Distribution>>,
        is_distribution_available: this.txFromJSON<boolean>,
        get_next_distribution_time: this.txFromJSON<u64>,
        time_before_next_distribution: this.txFromJSON<u64>
  }
}