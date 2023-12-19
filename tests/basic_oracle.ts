// eslint-disable-next-line node/no-unpublished-import
import type { USDY_USDC_ORACLE } from "../target/types/srfx_usdc_oracle";

import { MRENCLAVE, printLogs, setupTest, unixTimestamp } from "./utils";

import type { Program } from "@coral-xyz/anchor";
import * as anchor from "@coral-xyz/anchor";
import { sleep } from "@switchboard-xyz/common";
import type { FunctionAccount } from "@switchboard-xyz/solana.js";
import { SwitchboardWallet } from "@switchboard-xyz/solana.js"; 
import {
  attestationTypes,
  type BootstrappedAttestationQueue,
} from "@switchboard-xyz/solana.js";

describe("BASICORACLE", () => {
  let switchboard: BootstrappedAttestationQueue;
  let wallet: SwitchboardWallet;
  let functionAccount: FunctionAccount;

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.USDY_USDC_ORACLE as Program<USDY_USDC_ORACLE>;

  console.log(`ProgramID: ${program.programId}`);

  const payer = (program.provider as anchor.AnchorProvider).publicKey;

  const programStatePubkey = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("USDY_USDC_ORACLE_V2")],
    program.programId
  )[0];
  console.log(`programStatePubkey: ${programStatePubkey}`);

  const oraclePubkey = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ORACLE_USDY_SEED")],
    program.programId
  )[0];
  console.log(`oraclePubkey: ${oraclePubkey}`);

  before(async () => {
    switchboard = await setupTest(program.provider as anchor.AnchorProvider);
    console.log(`Attestation Queue: ${switchboard.attestationQueue.publicKey}`);

    [wallet] = await SwitchboardWallet.create(
      switchboard.program,
      switchboard.attestationQueue.publicKey,
      payer,
      "USDY_USDC_ORACLEFunctionWallet",
      16
    );
    console.log(`wallet: ${wallet.publicKey}`);

    [functionAccount] =
      await switchboard.attestationQueue.account.createFunction(
        {
          name: "test function",
          metadata: "this function handles XYZ for my protocol",
          schedule: "15 * * * * *",
          container: "switchboardlabs/balancer-oracle-function",
          version: "latest",
          mrEnclave: MRENCLAVE,
          authority: programStatePubkey,
        },
        wallet
      );
    console.log(`functionAccount: ${functionAccount.publicKey}`);
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize()
      .accounts({
        program: programStatePubkey,
        oracle: oraclePubkey,
        authority: payer,
        payer: payer,
        switchboardFunction: functionAccount.publicKey,
      })
      .rpc()
      .catch((err) => {
        console.error(err);
        throw err;
      });
    console.log("Your transaction signature", tx);
  });

  // it("Adds an enclave measurement", async () => {
  //   // Add your test here.
  //   const tx = await program.methods
  //     .setEnclaves({ mrEnclaves: [Array.from(MRENCLAVE)] })
  //     .accounts({
  //       program: programStatePubkey,
  //       authority: payer,
  //     })
  //     .rpc()
  //     .catch((err) => {
  //       console.error(err);
  //       throw err;
  //     });
  //   console.log("Your transaction signature", tx);
  //   const programState = await program.account.myProgramState.fetch(
  //     programStatePubkey
  //   );
  // });

  it("Oracle refreshes the prices", async () => {
    const securedSigner = anchor.web3.Keypair.generate();

    const rewardAddress =
      await switchboard.program.mint.getOrCreateAssociatedUser(payer);

    const functionState = await functionAccount.loadData();

    // TODO: generate function verify ixn
    const functionVerifyIxn = attestationTypes.functionVerify(
      switchboard.program,
      {
        params: {
          observedTime: new anchor.BN(unixTimestamp()),
          nextAllowedTimestamp: new anchor.BN(unixTimestamp() + 100),
          isFailure: false,
          mrEnclave: Array.from(MRENCLAVE),
        },
      },
      {
        function: functionAccount.publicKey,
        functionEnclaveSigner: securedSigner.publicKey,
        verifier: switchboard.verifier.publicKey,
        verifierSigner: switchboard.verifier.signer.publicKey,
        attestationQueue: switchboard.attestationQueue.publicKey,
        escrowWallet: functionState.escrowWallet,
        escrowTokenWallet: functionState.escrowTokenWallet,
        receiver: rewardAddress,
        verifierPermission: switchboard.verifier.permissionAccount.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      }
    );

    // Add your test here.
    const tx = await program.methods
      .refreshOracles({
        rows: [
          {
            symbol: { btc: {} },
            data: {
              oracleTimestamp: new anchor.BN(unixTimestamp()),
              price: new anchor.BN("25225000000000"), // 25225
              volume1hr: new anchor.BN("25225000000000"), // 1337000
              volume24hr: new anchor.BN("25225000000000"), // 1337000
              twap1hr: new anchor.BN("25225000000000"), // 25550
              twap24hr: new anchor.BN("25225000000000"), // 25550
            },
          },
          {
            symbol: { eth: {} },
            data: {
              oracleTimestamp: new anchor.BN(unixTimestamp()),
              price: new anchor.BN("1750000000000"), // 1750
              volume1hr: new anchor.BN("420000000000"), // 420000
              volume24hr: new anchor.BN("420000000000"), // 420000
              twap1hr: new anchor.BN("1750000000000"), // 1750
              twap24hr: new anchor.BN("1750000000000"), // 1750
            },
          },
        ],
      })
      .accounts({
        program: programStatePubkey,
        oracle: oraclePubkey,
        switchboardFunction: functionAccount.publicKey,
        enclaveSigner: securedSigner.publicKey,
      })
      .preInstructions([functionVerifyIxn])
      .signers([switchboard.verifier.signer, securedSigner])
      .rpc({ skipPreflight: true });

    console.log("Your transaction signature", tx);

    await printLogs(switchboard.program.connection, tx ? tx : "");

    await sleep(5000);

    const oracleState = await program.account.myOracleState.fetch(oraclePubkey);

    console.log(oracleState);

    console.log(`BTC\n`);
    printData(oracleState.btc);
    console.log(`ETH\n`);
    printData(oracleState.eth);
    console.log(`SOL\n`);
    printData(oracleState.sol);
  });
});

function normalizeDecimals(value: anchor.BN) {
  return (value ?? new anchor.BN(0))
    .div(new anchor.BN(10).pow(new anchor.BN(9)))
    .toNumber();
}

function printData(obj: {
  oracleTimestamp: anchor.BN;
  price: anchor.BN;
  volume1hr: anchor.BN;
  volume24hr: anchor.BN;
  twap1hr: anchor.BN;
  twap24hr: anchor.BN;
}) {
  console.log(`\tprice: ${normalizeDecimals(obj.price)}`);
  console.log(`\ttimestamp: ${obj.oracleTimestamp.toNumber()}`);
  console.log(`\t1Hr Volume: ${normalizeDecimals(obj.volume1hr)}`);
  console.log(`\t24Hr Volume: ${normalizeDecimals(obj.volume24hr)}`);
  console.log(`\t1Hr Twap: ${normalizeDecimals(obj.twap1hr)}`);
  console.log(`\t24Hr Twap: ${normalizeDecimals(obj.twap24hr)}`);
}
