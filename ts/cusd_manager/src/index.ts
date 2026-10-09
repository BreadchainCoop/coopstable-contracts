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




export type DataKey = {tag: "Owner", values: void} | {tag: "Admin", values: void} | {tag: "Manager", values: void} | {tag: "Cusd", values: void} | {tag: "YieldController", values: void} | {tag: "CusdSupply", values: void};

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

export interface Client {
  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a burn_cusd transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  burn_cusd: ({from, amount}: {from: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a issue_cusd transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_cusd: ({to, amount}: {to: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_cusd_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_cusd_id: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a set_cusd_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_cusd_id: ({new_cusd_id}: {new_cusd_id: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_cusd_issuer transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_cusd_issuer: ({new_issuer}: {new_issuer: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a cusd_total_supply transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cusd_total_supply: (options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a set_yield_controller transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_yield_controller: ({new_controller}: {new_controller: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

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
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABgAAAAAAAAAAAAAABU93bmVyAAAAAAAAAAAAAAAAAAAFQWRtaW4AAAAAAAAAAAAAAAAAAAdNYW5hZ2VyAAAAAAAAAAAAAAAABEN1c2QAAAAAAAAAAAAAAA9ZaWVsZENvbnRyb2xsZXIAAAAAAAAAAAAAAAAKQ3VzZFN1cHBseQAA",
        "AAAABAAAAK1FcnJvciBjb2RlcyBmb3IgdGhlIGN1c2RfbWFuYWdlciBjb250cmFjdC4gQ29tbW9uIGVycm9ycyBhcmUgY29kZXMgdGhhdCBtYXRjaCB1cCB3aXRoIHRoZSBidWlsdC1pbgpDVVNETWFuYWdlckVycm9yIGVycm9yIHJlcG9ydGluZy4gQ1VTRE1hbmFnZXIgc3BlY2lmaWMgZXJyb3JzIHN0YXJ0IGF0IDEwMAAAAAAAAAAAAAAQQ1VTRE1hbmFnZXJFcnJvcgAAAAYAAAAAAAAADUludGVybmFsRXJyb3IAAAAAAAABAAAAAAAAABdBbHJlYWR5SW5pdGlhbGl6ZWRFcnJvcgAAAAADAAAAAAAAABFVbmF1dGhvcml6ZWRFcnJvcgAAAAAAAAQAAAAAAAAAE05lZ2F0aXZlQW1vdW50RXJyb3IAAAAACAAAAAAAAAAMQmFsYW5jZUVycm9yAAAACgAAAAAAAAANT3ZlcmZsb3dFcnJvcgAAAAAAAAw=",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAAJYnVybl9jdXNkAAAAAAAAAgAAAAAAAAAEZnJvbQAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAA=",
        "AAAAAAAAAAAAAAAJc2V0X2FkbWluAAAAAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAKaXNzdWVfY3VzZAAAAAAAAgAAAAAAAAACdG8AAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAA=",
        "AAAAAAAAAAAAAAALZ2V0X2N1c2RfaWQAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAALc2V0X2N1c2RfaWQAAAAAAQAAAAAAAAALbmV3X2N1c2RfaWQAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAMAAAAAAAAAB2N1c2RfaWQAAAAAEwAAAAAAAAAFb3duZXIAAAAAAAATAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAPc2V0X2N1c2RfaXNzdWVyAAAAAAEAAAAAAAAACm5ld19pc3N1ZXIAAAAAABMAAAAA",
        "AAAAAAAAAAAAAAARY3VzZF90b3RhbF9zdXBwbHkAAAAAAAAAAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAAUc2V0X3lpZWxkX2NvbnRyb2xsZXIAAAABAAAAAAAAAA5uZXdfY29udHJvbGxlcgAAAAAAEwAAAAA=" ]),
      options
    )
  }
  public readonly fromJSON = {
    upgrade: this.txFromJSON<null>,
        burn_cusd: this.txFromJSON<null>,
        set_admin: this.txFromJSON<null>,
        issue_cusd: this.txFromJSON<null>,
        get_cusd_id: this.txFromJSON<string>,
        set_cusd_id: this.txFromJSON<null>,
        set_cusd_issuer: this.txFromJSON<null>,
        cusd_total_supply: this.txFromJSON<i128>,
        set_yield_controller: this.txFromJSON<null>
  }
}