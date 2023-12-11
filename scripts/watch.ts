import { SwitchboardProgram, loadKeypair } from "@switchboard-xyz/solana.js";
import * as anchor from "@coral-xyz/anchor";
import { UsdyUsdOracle } from "../target/types/usdy_usd_oracle";
import dotenv from "dotenv";
import { sleep } from "@switchboard-xyz/common";
import { PublicKey } from "@solana/web3.js";
import fs from 'fs'
dotenv.config();

(async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  let program = new anchor.Program(
    JSON.parse(
      fs.readFileSync(
        "./target/idl/usdy_usd_oracle.json",
        "utf8"
      ).toString()
    ),
    new PublicKey("8KVvnHxfz9xf3hvfD6Bpofcy2Rrqz9XgxZa66e9WEuvM"),
    provider
  );
  console.log(`PROGRAM: ${program.programId}`);

  const [programStatePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("USDY_USDC_ORACLE")],
    program.programId
  );
  console.log(`PROGRAM_STATE: ${programStatePubkey}`);
  const [oraclePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_USDY_SEED")],
    program.programId
  );
  console.log(`ORACLE_PUBKEY: ${oraclePubkey}`);

  let oracleState = await program.account.myOracleState.fetch(
    oraclePubkey
  );
  displayOracleState(oraclePubkey, oracleState as any); // apparently doesnt like _# syntax

  let lastFetched: number = Date.now();
  while (true) {
    await sleep(5000);
    oracleState = await program.account.myOracleState.fetch(oraclePubkey);
    console.log(oracleState)
    displayOracleState(oraclePubkey, oracleState as any); // apparently doesnt like _# syntax
  }
})();

interface OracleState {
  bump: number;
  usdyUsd: OracleData;
}
interface OracleData {
  oracleTimestamp: anchor.BN;
  mean: anchor.BN;
  median: anchor.BN;
  std: anchor.BN;
}
function displayOracleState(pubkey: PublicKey, oracleState: OracleState) {
  console.clear();
  console.log(`## Oracle (${pubkey})`);
  displaySymbol(oracleState.usdyUsd, "usdy_usd");
}

function displaySymbol(data: OracleData, symbol: string) {
  console.log(` > ${symbol.toUpperCase()} / USD`);
  console.log(`\Mean: $${new anchor.BN(data.mean.toString()).div(new anchor.BN(10 ** 9)).toNumber() / 10 ** 9}`);
  console.log(`\Median: $${new anchor.BN(data.median.toString()).div(new anchor.BN(10 ** 9)).toNumber() / 10 ** 9}`);
  console.log(`\Population Variance: ${new anchor.BN(data.std.toString()).div(new anchor.BN(10 ** 9)).toNumber() / 10 ** 9 * 100}%`);
}
