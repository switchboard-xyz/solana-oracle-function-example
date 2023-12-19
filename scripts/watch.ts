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
  
  const [ondoPriceFeedPubkey, _] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_USDY_SEED_V2"), new PublicKey("GBDDsAJHuKR6fJDv5aYj2bBPMbqdgxsaC87qcHpAXtcA").toBuffer(), Buffer.from("ondo_price_feed")],
    program.programId
  );
  const [ondoTradedFeedPubkey, __] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_USDY_SEED_V2"), new PublicKey("GBDDsAJHuKR6fJDv5aYj2bBPMbqdgxsaC87qcHpAXtcA").toBuffer(), Buffer.from("ondo_traded_feed")],
    program.programId
  );
  const ondoFeed = await program.account.aggregatorAccountData.fetch(ondoPriceFeedPubkey);
  const tradedFeed = await program.account.aggregatorAccountData.fetch(ondoTradedFeedPubkey);
  console.log(`ORACLE_PUBKEY_ONDO: ${ondoPriceFeedPubkey}`);
  console.log(`ORACLE_PUBKEY_TRADED: ${ondoTradedFeedPubkey}`);
  let oracleState = await program.account.myOracleState.fetch(
    oraclePubkey
  );

  displayOracleState(ondoPriceFeedPubkey, ondoFeed);
  displayOracleState(ondoTradedFeedPubkey, tradedFeed);

  let lastFetched: number = Date.now();
  while (true) {
    await sleep(5000);
    oracleState = await program.account.myOracleState.fetch(oraclePubkey);
    console.log(oracleState)
    displayOracleState(oraclePubkey, oracleState as any); // apparently doesnt like _# syntax
  }
})();

function displayOracleState(pubkey: PublicKey, oracleState: any) {
  console.log(`## Oracle (${pubkey})`);
  displaySymbol(oracleState.latestConfirmedRound, "usdy_usd");
}

function displaySymbol(data: AggregatorRound, symbol: string) {
  console.log(` > ${symbol.toUpperCase()} / USD`);
  if (data){
    console.log(`\tRound timestamp: ${data.roundOpenTimestamp}`);

    console.log(`\Price: ${data.result.mantissa.toNumber() / 10 ** data.result.scale}`);
  }
}
