import { SwitchboardProgram, loadKeypair } from "@switchboard-xyz/solana.js";
import * as anchor from "@coral-xyz/anchor";
import { UsdyUsdOracle } from "../target/types/usdy_usd_oracle";
import dotenv from "dotenv";
import { sleep } from "@switchboard-xyz/common";
import { PublicKey } from "@solana/web3.js";
import fs from 'fs'
dotenv.config();
import type { types } from "@switchboard-xyz/solana.js";
import { AggregatorAccount } from "@switchboard-xyz/solana.js";
import { AggregatorAccountData, AggregatorRound } from "@switchboard-xyz/solana.js/lib/generated";

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
    new PublicKey("2LuPhyrumCFRXjeDuYp1bLNYp7EbzUraZcvrzN9ZBUkN"),
    provider
  );
  console.log(`PROGRAM: ${program.programId}`);

  let switchboardProgram: SwitchboardProgram = await SwitchboardProgram.fromProvider(
    provider
  );

  
  const [programStatePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("USDY_USDC_ORACLE_V2")],
    program.programId
  );
  console.log(`PROGRAM_STATE: ${programStatePubkey}`);
  const [oraclePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_USDY_SEED_V2")],
    program.programId
  );
  console.log(`ORACLE_PUBKEY: ${oraclePubkey}`);
  
  const ondo = new PublicKey("2s52PZeGDJrA3pgD7ZSRBP8MdNyxrcVnyL4yBn39GQrj")
  const traded = new PublicKey("9npnQQpVLW7w3FdFDPGv4gJoxpKjJYAzjtQ2XxuJ8aE1")
  const ondoFeed = new AggregatorAccount(switchboardProgram, ondo);
  const ondoState: types.AggregatorAccountData =
    await ondoFeed.loadData();
  const tradedFeed = new AggregatorAccount(switchboardProgram, traded);
  const tradedState: types.AggregatorAccountData =
    await tradedFeed.loadData();
  console.log(`ORACLE_PUBKEY_ONDO: ${ondo}`);
  console.log(`ORACLE_PUBKEY_TRADED: ${traded}`);
  let oracleState = await program.account.myOracleState.fetch(
    oraclePubkey
  );

  displayOracleState(ondo, ondoState);
  displayOracleState(traded, tradedState);

  let lastFetched: number = Date.now();
  while (true) {
    await sleep(5000);
    oracleState = await program.account.myOracleState.fetch(oraclePubkey);
    console.log(oracleState)
    displayOracleState(oraclePubkey, oracleState as any); // apparently doesnt like _# syntax
  }
})();

function displayOracleState(pubkey: PublicKey, oracleState: AggregatorAccountData) {
  console.log(`## Oracle (${pubkey})`);
  displaySymbol(oracleState.latestConfirmedRound, "usdy_usd");
}

function displaySymbol(data: AggregatorRound, symbol: string) {
  console.log(` > ${symbol.toUpperCase()} / USD`);
  console.log(`\Price: ${data.result}`);
}
