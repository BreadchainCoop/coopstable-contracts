// sdk-probe.mts: run via sdk-probe.sh. Uses the yield-claimer's generated clients (stellar-sdk 14) against mainnet in simulation only. Never signs, never submits.
import * as YD from './yield_distributor.ts';
import * as YC from './lending_yield_controller.ts';
const rpcUrl = process.env.STELLAR_RPC_URL || 'https://mainnet.sorobanrpc.com';
const publicKey = 'GD2WECGE54BCB5K2D4UT37JEFZ3XK4NCMT6OMWPEFS5W2RKUYUW6WIDB';
const yd = new YD.Client({ contractId: 'CA2BNJFVCRJWCWLLOMJ3E3A62XMZ43U4K7RC6WVBOFC7RPYOJAH5BJVH', networkPassphrase: YD.Networks.PUBLIC, rpcUrl, publicKey });
const yc = new YC.Client({ contractId: 'CAIYIGHOUS5CG5MYNW4TPOW635ZLXIBEFFXJ6J4Z5ZBO3UC344VZRP3N', networkPassphrase: YC.Networks.PUBLIC, rpcUrl, publicKey });
const args = { protocol: 'BC_LA', asset: 'CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75' };

async function probe(name: string, fn: () => Promise<any>) {
  const out: any = { name };
  try {
    const tx = await fn();
    const sim = tx.simulation;
    out.simType = YD.rpc.Api.isSimulationError(sim) ? 'ERROR' : YD.rpc.Api.isSimulationRestore(sim) ? 'RESTORE' : YD.rpc.Api.isSimulationSuccess(sim) ? 'SUCCESS' : '?';
    out.minResourceFee = sim.minResourceFee;
    out.restorePreambleFee = sim.restorePreamble?.minResourceFee;
    out.error = sim.error?.slice(0, 160);
    try {
      const td = sim.transactionData?.build?.() ?? sim.transactionData;
      const xdr = td?.toXDR?.('base64');
      if (xdr) {
        const sd = YD.xdr.SorobanTransactionData.fromXDR(xdr, 'base64');
        const r = sd.resources();
        out.instructions = r.instructions();
        out.readBytes = r.diskReadBytes?.() ?? r.readBytes?.();
        out.writeBytes = r.writeBytes();
        out.footprintRO = r.footprint().readOnly().length;
        out.footprintRW = r.footprint().readWrite().length;
        const ext = sd.ext();
        out.archivedEntries = ext.switch?.() === 1 ? ext.v1?.()?.ext?.()?.v1?.()?.archivedSorobanEntries?.()?.length : 0;
      }
    } catch (e: any) { out.tdErr = String(e.message).slice(0, 120); }
    try { out.builtFeeStroops = tx.built?.fee; } catch {}
    try { const r = tx.result; out.result = typeof r === 'bigint' ? r.toString() : JSON.stringify(r, (_k, v) => typeof v === 'bigint' ? v.toString() : v)?.slice(0, 200); }
    catch (e: any) { out.resultThrows = String(e.message).slice(0, 200); }
  } catch (e: any) { out.threw = String(e.message).slice(0, 300); }
  console.log(JSON.stringify(out));
}

await probe('YD.is_distribution_available', () => yd.is_distribution_available());
await probe('YD.time_before_next_distribution', () => yd.time_before_next_distribution());
await probe('YD.get_distribution_info', () => yd.get_distribution_info());
await probe('YD.get_total_distributed', () => yd.get_total_distributed());
await probe('YC.get_yield', () => yc.get_yield(args));
await probe('YC.harvest_yield (simulate only)', () => yc.harvest_yield(args, { simulate: true }));
