#!/usr/bin/env python3
"""Rent of deployed contract-code entries, computed the way the network does.

    code_rent.py HASH|C... [...]      rent of each code entry, by hex wasm hash or by contract id (C...)
    code_rent.py --state              live Soroban state size, which sets the rent rate

size_for_rent = entry XDR bytes + wasm_module_memory_cost(cost_inputs)      soroban-env-host e2e_invoke.rs
memory cost   = sum over the ten ParseWasm* cost types of const + (linear * input >> 7)
fee (stroops) = ceil(size_for_rent * rate * ledgers / (1024 * 1215)) / 3     fees.rs, code entries only

Data entries pay their XDR size alone; the memory term exists only for code. The rate is 1,000 stroops per
KB per 1,215 ledgers while live state is below 2.0 GB (the floor), then -17,000 + 27,000 * state / 3 GB.
For a WASM that is not deployed yet: `stellar contract upload --build-only`, simulate, and read
minResourceFee (RPC simulation pads rent by ~15%).
"""
import argparse
import base64
import json
import struct
import urllib.request

INPUTS = ['n_instructions', 'n_functions', 'n_globals', 'n_table_entries', 'n_types',
          'n_data_segments', 'n_elem_segments', 'n_imports', 'n_exports', 'n_data_segment_bytes']
PARSE_WASM_INSTRUCTIONS = 23            # ContractCostType index; the ten ParseWasm* types are contiguous
CONTRACT_DATA, CONTRACT_CODE, CONFIG_SETTING = 6, 7, 8  # LedgerEntryType
SCV_CONTRACT_INSTANCE, SCV_LEDGER_KEY_CONTRACT_INSTANCE = 19, 20  # ScValType
LEDGER_COST, COST_PARAMS_MEMORY, STATE_SIZE_WINDOW = 2, 7, 12  # ConfigSettingID
RENT_RATE_FLOOR = 1000  # stroops per KB per 1,215 ledgers, observed on mainnet while state is below the knee


class Reader:
    def __init__(self, data):
        self.data, self.pos = data, 0

    def take(self, fmt):
        value = struct.unpack_from(fmt, self.data, self.pos)[0]
        self.pos += struct.calcsize(fmt)
        return value

    i32 = lambda self: self.take('>i')
    u32 = lambda self: self.take('>I')
    i64 = lambda self: self.take('>q')
    u64 = lambda self: self.take('>Q')


def ledger_entries(rpc_url, *keys):
    body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'getLedgerEntries',
                       'params': {'keys': [base64.b64encode(k).decode() for k in keys]}}).encode()
    req = urllib.request.Request(rpc_url, body, {'Content-Type': 'application/json', 'User-Agent': 'code_rent.py'})
    with urllib.request.urlopen(req) as res:
        return json.load(res)['result']['entries']


def config_setting(rpc_url, setting_id):
    entry = ledger_entries(rpc_url, struct.pack('>ii', CONFIG_SETTING, setting_id))[0]
    reader = Reader(base64.b64decode(entry['xdr']))
    assert reader.i32() == CONFIG_SETTING and reader.i32() == setting_id
    return reader


def memory_params(rpc_url):
    reader = config_setting(rpc_url, COST_PARAMS_MEMORY)
    params = []
    for _ in range(reader.u32()):
        reader.i32()  # ExtensionPoint
        params.append((reader.i64(), reader.i64()))
    return params[PARSE_WASM_INSTRUCTIONS:PARSE_WASM_INSTRUCTIONS + len(INPUTS)]


def rent_params(rpc_url):
    """(target bytes, rate at zero state, rate at target) from CONTRACT_LEDGER_COST_V0; the knee is 2/3 of target."""
    reader = config_setting(rpc_url, LEDGER_COST)
    reader.pos += 8 * 4  # the eight u32 limits
    fee_disk_read_entry, fee_write_entry, fee_read1kb, target, low, high = (reader.i64() for _ in range(6))
    return target, low, high


def rent_rate(rpc_url):
    target, low, high = rent_params(rpc_url)
    state = live_state_bytes(rpc_url)
    formula = low + (high - low) * state // target
    return state, target, formula, max(RENT_RATE_FLOOR, formula)


def live_state_bytes(rpc_url):
    reader = config_setting(rpc_url, STATE_SIZE_WINDOW)
    samples = [reader.u64() for _ in range(reader.u32())]
    return sum(samples) // len(samples)


def wasm_hash_of(rpc_url, contract_id):
    """Resolve a C... address to the hash of the WASM its instance currently runs."""
    raw_id = base64.b32decode(contract_id)[1:33]  # strkey: version byte, 32-byte id, checksum
    key = (struct.pack('>ii', CONTRACT_DATA, 1) + raw_id  # ScAddress::Contract
           + struct.pack('>ii', SCV_LEDGER_KEY_CONTRACT_INSTANCE, 1))  # key, durability persistent
    entries = ledger_entries(rpc_url, key)
    if not entries:
        raise SystemExit(f'{contract_id}: no contract instance on this network')
    reader = Reader(base64.b64decode(entries[0]['xdr']))
    assert reader.i32() == CONTRACT_DATA
    reader.pos += 4 + 4 + 32 + 4 + 4  # ext, ScAddress, key, durability
    assert reader.i32() == SCV_CONTRACT_INSTANCE
    if reader.i32() != 0:
        raise SystemExit(f'{contract_id}: not a WASM contract (Stellar asset contract)')
    return reader.data[reader.pos:reader.pos + 32].hex()


def code_entry(rpc_url, wasm_hash):
    entries = ledger_entries(rpc_url, struct.pack('>i', CONTRACT_CODE) + bytes.fromhex(wasm_hash))
    if not entries:
        raise SystemExit(f'{wasm_hash}: no such code entry')
    raw = base64.b64decode(entries[0]['xdr'])
    reader = Reader(raw)
    assert reader.i32() == CONTRACT_CODE
    inputs = None
    if reader.i32() == 1:  # ContractCodeEntry ext v1 carries the cost inputs
        reader.i32(), reader.i32()  # ExtensionPoints
        inputs = {name: reader.u32() for name in INPUTS}
    reader.pos += 32  # hash
    return len(raw), reader.u32(), inputs, entries[0].get('liveUntilLedgerSeq')


def rent(size_for_rent, rate, ledgers, is_code):
    fee = -(-size_for_rent * rate * ledgers // (1024 * 1215))
    return -(-fee // 3) if is_code else fee


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('hashes', nargs='*', help='hex wasm hashes or C... contract ids')
    parser.add_argument('--rpc', default='https://mainnet.sorobanrpc.com/')
    parser.add_argument('--rate', type=int, help='rent fee per KB per 1,215 ledgers; default: the network\'s current rate')
    parser.add_argument('--ledgers', type=int, default=2_073_600, help='TTL to price (default 120 days)')
    parser.add_argument('--state', action='store_true', help='print live Soroban state size and the resulting rate')
    args = parser.parse_args()

    state, target, formula, rate = rent_rate(args.rpc)
    if args.rate is None:
        args.rate = rate
    if args.state:
        print(f'live state {state / 1e9:.3f} GB of a {target / 1e9:.1f} GB target: rate {rate} stroops/KB/1215 ledgers '
              f'(formula {formula}, floor {RENT_RATE_FLOOR}, leaves the floor at {target * 2 / 3 / 1e9:.2f} GB)')

    if not args.hashes:
        return
    params = memory_params(args.rpc)
    total = 0
    for target in args.hashes:
        wasm_hash = wasm_hash_of(args.rpc, target) if target.startswith('C') else target
        if target != wasm_hash:
            print(f'{target} runs {wasm_hash}')
        xdr_size, wasm_size, inputs, live_until = code_entry(args.rpc, wasm_hash)
        parts = {name: c + (l * inputs[name] >> 7) for name, (c, l) in zip(INPUTS, params)} if inputs else {}
        memory = sum(parts.values())
        fee = rent(xdr_size + memory, args.rate, args.ledgers, True)
        total += fee
        print(f'{wasm_hash[:8]}  wasm {wasm_size:>6} B  memory {memory:>9} B  size_for_rent {xdr_size + memory:>9} B'
              f'  {fee / 1e7:7.2f} XLM / {args.ledgers} ledgers  liveUntil={live_until}')
        if inputs:
            breakdown = ', '.join(f'{name[2:]}={inputs[name]} ({parts[name]} B)' for name in INPUTS if parts[name])
            print(f'          {breakdown}')
    if len(args.hashes) > 1:
        print(f'total {total / 1e7:.2f} XLM / {args.ledgers} ledgers')


if __name__ == '__main__':
    main()
