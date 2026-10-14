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





export interface AssetEpochPrincipal {
  deposits_in_epoch: i128;
  epoch: u64;
  last_updated: u64;
  principal: i128;
  withdrawals: i128;
}


export interface Request {
  address: string;
  amount: i128;
  request_type: u32;
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

export type SupportedAdapter = {tag: "BlendCapital", values: void} | {tag: "Custom", values: readonly [string]};

export type SupportedYieldType = {tag: "Lending", values: void} | {tag: "Liquidity", values: void} | {tag: "Custom", values: readonly [string]};

export interface Client {
  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_apy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_apy: ({asset}: {asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * (Owner only) Replace the contract code; the address and all storage survive.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_yield: ({asset}: {asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a claim_yield transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_yield: ({asset, yield_amount}: {asset: string, yield_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a get_balance transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_balance: ({user, asset}: {user: string, asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a deposit_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_auth: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a get_emissions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_emissions: ({asset}: {asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a withdraw_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_auth: ({user, asset, amount}: {user: string, asset: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a protocol_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  protocol_token: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a claim_emissions transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_emissions: ({to, asset}: {to: string, asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a claim_yield_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_yield_auth: ({asset, amount}: {asset: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a get_total_deposited transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_deposited: ({asset}: {asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<i128>>

  /**
   * Construct and simulate a claim_emissions_auth transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_emissions_auth: ({to, asset}: {to: string, asset: string}, options?: MethodOptions) => Promise<AssembledTransaction<Option<readonly [string, string, Array<any>]>>>

  /**
   * Construct and simulate a update_epoch_principal transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_epoch_principal: ({asset, epoch, principal}: {asset: string, epoch: u64, principal: i128}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {yield_controller, blend_pool_id, blend_token_id, owner, asset}: {yield_controller: string, blend_pool_id: string, blend_token_id: string, owner: string, asset: string},
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
    return ContractClient.deploy({yield_controller, blend_pool_id, blend_token_id, owner, asset}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAAE0Fzc2V0RXBvY2hQcmluY2lwYWwAAAAABQAAAAAAAAARZGVwb3NpdHNfaW5fZXBvY2gAAAAAAAALAAAAAAAAAAVlcG9jaAAAAAAAAAYAAAAAAAAADGxhc3RfdXBkYXRlZAAAAAYAAAAAAAAACXByaW5jaXBhbAAAAAAAAAsAAAAAAAAAC3dpdGhkcmF3YWxzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAB1JlcXVlc3QAAAAAAwAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAADHJlcXVlc3RfdHlwZQAAAAQ=",
        "AAAAAwAAAAAAAAAAAAAAC1JlcXVlc3RUeXBlAAAAAAoAAAAAAAAABlN1cHBseQAAAAAAAAAAAAAAAAAIV2l0aGRyYXcAAAABAAAAAAAAABBTdXBwbHlDb2xsYXRlcmFsAAAAAgAAAAAAAAASV2l0aGRyYXdDb2xsYXRlcmFsAAAAAAADAAAAAAAAAAZCb3Jyb3cAAAAAAAQAAAAAAAAABVJlcGF5AAAAAAAABQAAAAAAAAAaRmlsbFVzZXJMaXF1aWRhdGlvbkF1Y3Rpb24AAAAAAAYAAAAAAAAAEkZpbGxCYWREZWJ0QXVjdGlvbgAAAAAABwAAAAAAAAATRmlsbEludGVyZXN0QXVjdGlvbgAAAAAIAAAAAAAAABhEZWxldGVMaXF1aWRhdGlvbkF1Y3Rpb24AAAAJ",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAAHZ2V0X2FweQAAAAABAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAABA==",
        "AAAAAAAAAEwoT3duZXIgb25seSkgUmVwbGFjZSB0aGUgY29udHJhY3QgY29kZTsgdGhlIGFkZHJlc3MgYW5kIGFsbCBzdG9yYWdlIHN1cnZpdmUuAAAAB3VwZ3JhZGUAAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAACw==",
        "AAAAAAAAAAAAAAAJZ2V0X3lpZWxkAAAAAAAAAQAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAALY2xhaW1feWllbGQAAAAAAgAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAx5aWVsZF9hbW91bnQAAAALAAAAAQAAAAs=",
        "AAAAAAAAAAAAAAALZ2V0X2JhbGFuY2UAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAAMZGVwb3NpdF9hdXRoAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+gAAAPtAAAAAwAAABMAAAARAAAD6gAAAAA=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAUAAAAAAAAAEHlpZWxkX2NvbnRyb2xsZXIAAAATAAAAAAAAAA1ibGVuZF9wb29sX2lkAAAAAAAAEwAAAAAAAAAOYmxlbmRfdG9rZW5faWQAAAAAABMAAAAAAAAABW93bmVyAAAAAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAANZ2V0X2VtaXNzaW9ucwAAAAAAAAEAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAEAAAAL",
        "AAAAAAAAAAAAAAANd2l0aGRyYXdfYXV0aAAAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAVhc3NldAAAAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPoAAAD7QAAAAMAAAATAAAAEQAAA+oAAAAA",
        "AAAAAAAAAAAAAAAOcHJvdG9jb2xfdG9rZW4AAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAPY2xhaW1fZW1pc3Npb25zAAAAAAIAAAAAAAAAAnRvAAAAAAATAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAACw==",
        "AAAAAAAAAAAAAAAQY2xhaW1feWllbGRfYXV0aAAAAAIAAAAAAAAABWFzc2V0AAAAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+gAAAPtAAAAAwAAABMAAAARAAAD6gAAAAA=",
        "AAAAAAAAAAAAAAATZ2V0X3RvdGFsX2RlcG9zaXRlZAAAAAABAAAAAAAAAAVhc3NldAAAAAAAABMAAAABAAAACw==",
        "AAAAAAAAAAAAAAAUY2xhaW1fZW1pc3Npb25zX2F1dGgAAAACAAAAAAAAAAJ0bwAAAAAAEwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAQAAA+gAAAPtAAAAAwAAABMAAAARAAAD6gAAAAA=",
        "AAAAAAAAAAAAAAAWdXBkYXRlX2Vwb2NoX3ByaW5jaXBhbAAAAAAAAwAAAAAAAAAFYXNzZXQAAAAAAAATAAAAAAAAAAVlcG9jaAAAAAAAAAYAAAAAAAAACXByaW5jaXBhbAAAAAAAAAsAAAAA",
        "AAAAAgAAAAAAAAAAAAAAEFN1cHBvcnRlZEFkYXB0ZXIAAAACAAAAAAAAAAAAAAAMQmxlbmRDYXBpdGFsAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==",
        "AAAAAgAAAAAAAAAAAAAAElN1cHBvcnRlZFlpZWxkVHlwZQAAAAAAAwAAAAAAAAAAAAAAB0xlbmRpbmcAAAAAAAAAAAAAAAAJTGlxdWlkaXR5AAAAAAAAAQAAAAAAAAAGQ3VzdG9tAAAAAAABAAAAEQ==" ]),
      options
    )
  }
  public readonly fromJSON = {
    deposit: this.txFromJSON<i128>,
        get_apy: this.txFromJSON<u32>,
        upgrade: this.txFromJSON<null>,
        withdraw: this.txFromJSON<i128>,
        get_yield: this.txFromJSON<i128>,
        claim_yield: this.txFromJSON<i128>,
        get_balance: this.txFromJSON<i128>,
        deposit_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        get_emissions: this.txFromJSON<i128>,
        withdraw_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        protocol_token: this.txFromJSON<string>,
        claim_emissions: this.txFromJSON<i128>,
        claim_yield_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        get_total_deposited: this.txFromJSON<i128>,
        claim_emissions_auth: this.txFromJSON<Option<readonly [string, string, Array<any>]>>,
        update_epoch_principal: this.txFromJSON<null>
  }
}