import { SwitchboardProgram, loadKeypair } from "@switchboard-xyz/solana.js";
import * as anchor from "@coral-xyz/anchor";
import { UsdyUsdOracle } from "../target/types/usdy_usd_oracle";
import dotenv from "dotenv";
import { loadDefaultQueue } from "./utils";
import fs from 'fs'
import { PublicKey } from "@solana/web3.js";
dotenv.config();

(async () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(
    process.argv.length > 2
      ? new anchor.AnchorProvider(
          provider.connection,
          new anchor.Wallet(loadKeypair(process.argv[2])),
          {}
        )
      : provider
  );

  const payer = (provider.wallet as anchor.Wallet).payer;
  console.log(`PAYER: ${payer.publicKey}`);

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

  const switchboardProgram = await SwitchboardProgram.fromProvider(provider);

  const [programStatePubkey, b1] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("USDY_USDC_ORACLE")],
    program.programId
  );
  console.log(`PROGRAM_STATE: ${programStatePubkey}`);

  const [oraclePubkey, b2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_USDY_SEED")],
    program.programId
  );
  console.log(`ORACLE_PUBKEY: ${oraclePubkey}`);

  try {
    const programState = await program.account.myProgramState.fetch(
      programStatePubkey
    );
    console.log(`Program state already initialized`);
    console.log(
      `PROGRAM_STATE: \n${JSON.stringify(programState, undefined, 2)}`
    );
    
    const oracleState = await program.account.myOracleState.fetch(
      oraclePubkey
    );
    console.log(
      `ORACLE_STATE: \n${JSON.stringify(oracleState, undefined, 2)}`
    );
    
    return;

    // Account already initialized
  } catch (error) {
    if (!`${error}`.includes("Account does not exist or has no data")) {
      throw error;
    }
  }

  const attestationQueueAccount = await loadDefaultQueue(switchboardProgram);
  console.log(`ATTESTATION_QUEUE: ${attestationQueueAccount.publicKey}`);

  // Create the instructions to initialize our Switchboard Function
  const [functionAccount, functionInit] =
    await attestationQueueAccount.createFunctionInstruction(payer.publicKey, {
      container: `${process.env.DOCKERHUB_ORGANIZATION ?? "switchboardlabs"}/${
        process.env.DOCKERHUB_CONTAINER_NAME ?? "solana-balancer-oracle-function"
      }`,
      version: `${process.env.DOCKERHUB_CONTAINER_VERSION ?? "latest"}`, // TODO: set to 'latest' after testing
    });
  console.log(`SWITCHBOARD_FUNCTION: ${functionAccount.publicKey}`);

  const signature = await program.methods
    .initialize(b1, b2)
    .accounts({
      oracle: oraclePubkey,
      program: programStatePubkey,
      authority: payer.publicKey,
      payer: payer.publicKey,
      switchboardFunction: functionAccount.publicKey,
    })
    .signers([...functionInit.signers])
    .preInstructions([...functionInit.ixns])
    .rpc();

  console.log(`[TX] initialize: ${signature}`);
})();