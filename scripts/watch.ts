import { SwitchboardProgram, loadKeypair } from "@switchboard-xyz/solana.js";
import * as anchor from "@coral-xyz/anchor";
import { TwapOracle } from "../target/types/twap_oracle";
import dotenv from "dotenv";
import { sleep } from "@switchboard-xyz/common";
import { PublicKey } from "@solana/web3.js";
dotenv.config();

(async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program: anchor.Program<TwapOracle> = anchor.workspace.TwapOracle;
  console.log(`PROGRAM: ${program.programId}`);

  const [programStatePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("TWAPORACLE")],
    program.programId
  );
  console.log(`PROGRAM_STATE: ${programStatePubkey}`);
  const programState = await program.account.myProgramState.fetch(
    programStatePubkey
  );

  const [oraclePubkey] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_V1_SEED")],
    program.programId
  );
  console.log(`ORACLE_PUBKEY: ${oraclePubkey}`);

  let oracleState = await program.account.myOracleState.fetch(oraclePubkey);
  let lastFetched: number = Date.now();
  while (true) {
    await sleep(5000);
    oracleState = await program.account.myOracleState.fetch(oraclePubkey);
    displayOracleState(oraclePubkey, oracleState as any); // apparently doesnt like _# syntax
  }
})();

interface OracleState {
  bump: number;
  Srfx_usdc: OracleData;
}
interface OracleData {
  oracleTimestamp: anchor.BN;
  price: anchor.BN;
}
function displayOracleState(pubkey: PublicKey, oracleState: OracleState) {
  console.clear();
  console.log(`## Oracle (${pubkey})`);
  displaySymbol(oracleState.Srfx_usdc, "srfx_usdc");
}

function displaySymbol(data: OracleData, symbol: string) {
  console.log(` > ${symbol.toUpperCase()} / USD`);
  console.log(`\tPrice: ${data.price}`);
}
