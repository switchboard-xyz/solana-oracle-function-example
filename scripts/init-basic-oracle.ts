import { QueueAccount, SwitchboardProgram, loadKeypair } from "@switchboard-xyz/solana.js";
import * as anchor from "@coral-xyz/anchor";
import { UsdyUsdOracle } from "../target/types/usdy_usd_oracle";
import dotenv from "dotenv";
import { loadDefaultQueue } from "./utils";
import fs from 'fs'
import { Connection, PublicKey } from "@solana/web3.js";
import type {
  JobAccount,
} from "@switchboard-xyz/solana.js";
import { OracleAccount } from "@switchboard-xyz/solana.js";
import { OracleJob } from "@switchboard-xyz/common";

dotenv.config();

(async () => {

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);



  const payer = (provider.wallet as anchor.Wallet).payer;
  console.log(`PAYER: ${payer.publicKey}`);

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

  const switchboardProgram = await SwitchboardProgram.fromProvider(provider);

  const [programStatePubkey, b1] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("USDY_USDC_ORACLE_V2")],
    program.programId
  );
  console.log(`PROGRAM_STATE: ${programStatePubkey}`);

  
let switchboard: SwitchboardProgram = await SwitchboardProgram.fromProvider(
  provider
);

const queueAccount = new QueueAccount(
  switchboard,
  "uPeRMdfPmrPqgRWSrjAnAkH78RqAhe5kXoW6vBYRqFX"
); // devnet
const [oracle, b2] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("ORACLE_USDY_SEED_V2")],
  program.programId
);
console.log(`ORACLE_PUBKEY: ${oracle}`);



  const attestationQueueAccount = await loadDefaultQueue(switchboardProgram);
  console.log(`ATTESTATION_QUEUE: ${attestationQueueAccount.publicKey}`);

  // Create the instructions to initialize our Switchboard Function
  const [functionAccount, functionInit] =
    await attestationQueueAccount.createFunctionInstruction(payer.publicKey, {
      container: `${process.env.DOCKERHUB_ORGANIZATION ?? "switchboardlabs"}/${
        process.env.DOCKERHUB_CONTAINER_NAME ?? "solana-ondo-oracle-function"
      }`,
      version: `${process.env.DOCKERHUB_CONTAINER_VERSION ?? "latest"}`, // TODO: set to 'latest' after testing
    });
  console.log(`SWITCHBOARD_FUNCTION: ${functionAccount.publicKey}`);

/*
  const signature = await program.methods
    .initialize(b1, b2) //initialize 
    .accounts({
      oracle,
      program: programStatePubkey,
      authority: payer.publicKey,
      payer: payer.publicKey,
      switchboardFunction: new PublicKey("DSYtAsC1YAfMBDoJPAd2Q5rDEwKnrBkzN1wNQ7aYd1o5")//functionAccount.publicKey,
    })
    .signers([...functionInit.signers])
    .preInstructions([...functionInit.ixns])
    .rpc();

  console.log(`[TX] initialize: ${signature}`);
await provider.connection.confirmTransaction(signature, "confirmed");*/
const [ondoFeed] = await queueAccount.createFeed({
  batchSize: 1,
  minRequiredOracleResults: 1,
  minRequiredJobResults: 1,
  minUpdateDelaySeconds: 5,
  fundAmount: 0.38,
  
  name: "ondo-price-feed",
  enable: true,
  crankPubkey: new PublicKey("UcrnK4w2HXCEjY8z6TcQ9tysYr3c9VcFLdYAU9YQP5e"),
  
  jobs: [
    // existing job account
    // or create a new job account with the feed
    {
      weight: 2,
      data: OracleJob.encodeDelimited(
        OracleJob.fromObject({
          tasks: [
            {
              solanaAccountDataFetchTask: {

                pubkey: oracle.toBase58(),
              }
            },{
              bufferLayoutParseTask: {
                endian: OracleJob.BufferLayoutParseTask.Endian.LITTLE_ENDIAN,
                type: OracleJob.BufferLayoutParseTask.BufferParseType.u64,
                offset: 1+8+8
              },

            },
            {
              divideTask: {
                big: 1000000000,
              },
            },
          ],
        })
      ).finish(),
    },
  ],
});

const [tradedFeed] = await queueAccount.createFeed({
  batchSize: 1,
  minRequiredOracleResults: 1,
  minRequiredJobResults: 1,
  minUpdateDelaySeconds: 5,
  fundAmount: 0.38,
  enable: true,
  name: "ondo-traded-price-feed",
  crankPubkey: new PublicKey("UcrnK4w2HXCEjY8z6TcQ9tysYr3c9VcFLdYAU9YQP5e"),
  
  jobs: [
    // existing job account
    // or create a new job account with the feed
    {
      weight: 2,
      data: OracleJob.encodeDelimited(
        OracleJob.fromObject({
          tasks: [
            {
              solanaAccountDataFetchTask: {

                pubkey: oracle.toBase58(),
              }},
              {
              bufferLayoutParseTask: {
                endian: OracleJob.BufferLayoutParseTask.Endian.LITTLE_ENDIAN,
                type: OracleJob.BufferLayoutParseTask.BufferParseType.u64,
                offset: 1+8+8+8 
              },

            },
            // divide 1000000000
            {
              divideTask: {
                big: 1000000000,
              },
            },
          ],
        })
      ).finish(),
    },
  ],
});
console.log(`ORACLE_PUBKEY_ONDO: ${ondoFeed?.publicKey}`);
console.log(`ORACLE_PUBKEY_TRADED: ${tradedFeed?.publicKey}`);

})();