# Overview

## Objective

We want to adjust a sample Switchboard program and function to grab the price of SRFX vs USD, instead of the current functionality of the example. Luckily, the example does nearly all the work for us (nice) and the team have made some further stuff abstracted that makes this (much, much) easier than it used to be - ie. `switchboard_utils::exchanges::*Api::fetch_ticker`. Phew!

fork and clone https://github.com/switchboard-xyz/solana-oracle-function-example!

balancer.rs

it's not aptly named, as we're pulling data from onchain balancer pools. Renamne it to balancer.rs.

**Technical Documentation:**

**Objective**: Update the `balancer.rs` file in a Rust project to modify the data structures and fetching logic for a cryptocurrency oracle.

**Before You Start:**

- Ensure you have a text editor installed (e.g., Visual Studio Code).
- Ensure you have access to the project containing the `balancer.rs` file.
- Understand that we are modifying how data is fetched and structured in a Rust program.

**Step-by-Step Instructions:**

1. Open the project in your text editor.
2. Navigate to `src/balancer.rs` within the `02_srfx_usdc_oracle_function_rust` directory of the project.
3. Find the section that imports modules at the top of the file.
4. Change `use BASICORACLE::{OracleDataBorsh, TradingSymbol};` to `use srfx_usdc_oracle::{OracleDataBorsh, TradingSymbol};`.
5. Scroll to the `Ticker` struct definition section.
6. Remove the unused fields such as `priceChange`, `priceChangePercent`, `weightedAvgPrice`, `openPrice`, `highPrice`, `lowPrice`, `lastPrice`, `volume`, `quoteVolume`, `openTime`, and `closeTime`.
7. Change `pub lastPrice: String,` to `pub price: I256,`.
8. Locate the `IndexData` struct and modify the fields to only include `pub symbol: String` and `pub quote: Ticker`.
9. Adjust the `Into<OracleDataBorsh>` implementation for `IndexData` to only convert the new `price` field from the `Ticker` struct.
10. Update the `Balancer` struct to contain only one `IndexData` field named `pub srfx_usdc`.
11. Adjust the `Balancer` implementation\'s `fetch` method to set up the new field based on the symbol `"USDYUSD"` and the provided `price`.
12. change the line inside `impl Into<OracleDataBorsh> for IndexData {` to `price: self.data.price.as_u64()`

lib.rs

Let's apply the changes to the file `lib.rs`. These changes involve identifiers and program seeds in a Rust programming project.

**Step-by-Step Instructions for Beginners:**

1. **Find Identifiers:**
   - Original Identifier: `3NKUtPKboaQN4MwY3nyULBesFaW7hHsXFrBTVjbn2nBr`
   - New Identifier: `2X2ocABrA21nKv87MT2n71kZKv6DsFmxv3b9mTTS4AET`

2. **Update Program Seed:**
   - Original Seed: `BASICORACLE`
   - New Seed: `USDY_USDC_ORACLE`


Now let's talk about changes to `models.rs`.

**Step-by-Step Instructions for Beginners:**

1. **Simplify `OracleData` Structures:**
   - Remove unnecessary data: `volume_1hr, volume_24hr, twap_1hr, twap_24hr` from both `OracleData` and `OracleDataBorsh`.

2. **Update `MyOracleState` To Reflect New Data:**
   - Change multiple oracle data tracking like `btc, usdc, eth, sol, doge` to just `srfx_usdc`.

3. **Update `price` To Use Smaller Type:**
    - ensure each price is `price: u64`.

main.rs

Here is how we will do it, step by step:

1. First, open the `main.rs` file in a text editor that lets you edit code, like Visual Studio Code or Sublime Text.
2. Start with the existing code and prepare to add new ones.
3. We\'re adding new tools for our recipe to work with. Copy and paste the following code into your `main.rs` file at the very top:
   ```rust
   use std::future::Future;
   use std::pin::Pin;
   use std::boxed::Box;
   ```

4. Next, paste these code right after the ones you just added:
   ```rust
   use rust_decimal::Decimal;
   use ethers::{
      providers::{Http, Provider},
   };
   ```

5. Paste these new lines as well:
   ```rust
   pub const PROGRAM_SEED: &[u8] = b"USDY_USDC_ORACLE";
   ```

6. I'm not sure we should worry stepping thru these helper functions, as they will become standard in switchboard_utils as time goes on. If you have any specific issues or face any questions, feel free to reach me at @staccoverflow and we can work thru them!

```


pub async fn fetch_balancer_v2_quote(rpc_url: &str, pool: PoolId, in_token: &str, out_token: &str, amount: U256, slippage: f64) -> Result<U256> {
    let private_key = PrivateKey::from_str("00e0000a00aaaa0e0a000e0e0000e00e000a000000000000000aaa00a0aaaaaa").unwrap();
    let w3 = build_web3(rpc_url);
    let vault_instance = balancer_sdk::vault::Vault::new(w3);
    let in_token = addr!(in_token);
    let out_token = addr!(out_token);
    let pool_info = vault_instance.get_pool_tokens(pool.into()).call().await.unwrap();
    // println!("{:#?}", pool_info);
    let out_idx = pool_info.0.iter().position(|&x| x == out_token).unwrap();
    let in_idx = pool_info.0.iter().position(|&x| x == in_token).unwrap();
    let in_pool_total = pool_info.1[in_idx];
    let out_pool_total = pool_info.1[out_idx];
    let swap_step = BatchSwapStep {
        pool_id: pool,
        asset_in_index: in_idx,
        asset_out_index: out_idx,
        amount,
        user_data: UserData("0x").into(),
    };
    let funds = FundManagement {
        sender: private_key.public_address(),
        from_internal_balance: false,
        recipient: private_key.public_address(),
        to_internal_balance: false,
    };
    let deltas = vault_instance
        .query_batch_swap(
            SwapKind::GivenIn as u8,
            vec![swap_step.into()],
            pool_info.0,
            funds.into(),
        )
        .from(balancer_sdk::Account::Offline(private_key, None))
        .call()
        .await
        .unwrap();
    // println!("{}", deltas[out_idx].abs());
    let in_percent = get_percentage_of_total(amount, in_pool_total);
    let out_amount = deltas[out_idx].abs().try_into().unwrap();
    let out_percent = get_percentage_of_total(out_amount, out_pool_total);
    if in_percent >= slippage {
        println!("IN CHANGE: {}%", in_percent);
        return Err(Box::new(SbError::CustomMessage("IN CHANGE".to_string())));
    }
    if out_percent >= slippage {
        println!("OUT CHANGE: {}%", out_percent);
        return Err(Box::new(SbError::CustomMessage("OUT CHANGE".to_string())));
    }
    Ok(out_amount)
}

fn median(mut numbers: Vec<Decimal>) -> Option<balancer_sdk::I256> {

    if numbers.is_empty() {
        return None; // Cannot find the median of an empty list
    }

    // Sort the numbers using `partial_cmp` because `Decimal` doesn't implement `Ord`
    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let mid = numbers.len() / 2; // Find the middle index
    Some(if numbers.len() % 2 == 0 {
        // If even number of elements, average the middle two
        let decimal = (numbers[mid - 1] + numbers[mid]) / Decimal::from(2);
        let i256 = (balancer_sdk::I256::from(decimal.to_u128().unwrap()));
        i256
    } else {
        // If odd, return the middle element
        let decimal = numbers[mid];
        let i256 = (balancer_sdk::I256::from(decimal.to_u128().unwrap()));
        i256
    })
}

fn u256_to_f64(value: U256) -> f64 {
    // This function assumes that the value can fit within an f64
    // It's a simple way to convert, and more sophisticated methods may be needed for larger values
    value.as_u128() as f64
}

fn get_percentage_of_total(part: U256, total: U256) -> f64 {
    if total.is_zero() {
        panic!("Total must not be zero");
    }

    // Convert both U256 values to f64
    let part = u256_to_f64(part);
    let total = u256_to_f64(total);

    // Calculate the percentage
    (part / total) * 100.0
}

```

7. The 'perform' function has changed significantly. Let's step through it:

```
    // Then, write your own Rust logic and build a Vec of instructions.
    // Should  be under 700 bytes after serialization
```
Pulling data from Arbitrum Goerli, cuz that's where the data lies:
```
    // setup the provider + signer
    let provider = Provider::<Http>::try_from("https://goerli-rollup.arbitrum.io/rpc").unwrap();

    let SFRXETH_WSTETH_POOL = pool_id!("0x42ed016f826165c2e5976fe5bc3df540c5ad0af700000000000000000000058b");
    let WSTETH_WETH_POOL = pool_id!("0x93d199263632a4ef4bb438f1feb99e57b4b5f0bd0000000000000000000005c2");
    // let WETH_USDC_POOL_UNSAFE = pool_id!("0x96646936b91d6b9d7d0c47c496afbf3d6ec7b6f8000200000000000000000019");
    // https://ethereumnodes.com/
    let rpc_url = "https://rpc.flashbots.net/";

    let pool_id = SFRXETH_WSTETH_POOL;
```
Get the srfx prices from Balancer:
```
    let wsteth_amount = fetch_balancer_v2_quote(rpc_url, SFRXETH_WSTETH_POOL, SFRX_ETH, WST_ETH, u256!(10u128.pow(SFRXETH_DECIMALS).to_string()), 0.1).await.unwrap();
    let weth_amount = fetch_balancer_v2_quote(rpc_url, WSTETH_WETH_POOL, WST_ETH, WETH, wsteth_amount, 0.1).await.unwrap();
```
Get the Eth prices from an array of sources, then grab the average:
```
    // let usdc_amount = fetch_balancer_v2_quote(rpc_url, WETH_USDC_POOL_UNSAFE, WETH, USDC, weth_amount, 0.1).await?;
    let bitfinex_quote = switchboard_utils::exchanges::BitfinexApi::fetch_ticker("tETHUSD", None).await.unwrap();
    let coinbase_quote = switchboard_utils::exchanges::CoinbaseApi::fetch_ticker("ETH-USD", None).await.unwrap();
    let huobi_quote = switchboard_utils::exchanges::HuobiApi::fetch_ticker("ethusdt", None).await.unwrap();
    let kraken_quote = switchboard_utils::exchanges::KrakenApi::fetch_ticker("ETHUSD", None).await.unwrap();
    println!("Bitfinex: {}", bitfinex_quote.last_price);
    println!("Coinbase: {}", coinbase_quote.price);
    println!("Huobi: {}", huobi_quote.close);
    println!("Kraken: {}", kraken_quote.close[0]);
    let eth_prices = vec![
        bitfinex_quote.last_price,
        coinbase_quote.price,
        Decimal::from_f64(huobi_quote.close).unwrap(),
        kraken_quote.close[0],
    ];
    let eth_price = median(eth_prices).unwrap();
    println!("ETH Price: {}", eth_price);
    println!("WETH Amount: {}", weth_amount);
```
Summmon the Balancer file to first save and then build the ixns for this perform function to then emit. What a sentence that was!
```
    msg!("sending transaction");

    // Finally, emit the signed quote and partially signed transaction to the functionRunner oracle
    // The functionRunner oracle will use the last outputted word to stdout as the serialized result. This is what gets executed on-chain.
    println!("{} {}", eth_price, ((eth_price * I256::from_dec_str(weth_amount.to_string().as_str()).unwrap())).to_string().as_str());
    let balancer = Balancer::fetch((eth_price * I256::from_dec_str(weth_amount.to_string().as_str()).unwrap())).await?;
    println!("{} {}", balancer.srfx_usdc.quote.price, TradingSymbol::Srfx_usdc as u8);
    let ixs: Vec<Instruction> = balancer.to_ixns(&runner);
    runner.emit(ixs).await?;
    Ok(())

```
8. we're constrained on execution time within the sgx enclave, so let's remove the extra `if` block near the top containing `runner.emit_error(199).await.unwrap();`

Great job! You’ve just updated a computer program.

We've got some more changes to make in our project, and I'm here to walk you through them, step by step. Cool? Cool.

**Part 1: Update `.env` File**

1. Open the file named `.env`.
2. You'll see something like `DOCKERHUB_ORGANIZATION=switchboardlabs`.
3. Change `switchboardlabs` to `your own docker username`.
4. That's it for this file.

**Part 2: Update `Anchor.toml` File**

1. Locate the file `Anchor.toml`.
2. It's like a list of settings. Find the section `[programs.localnet]` and `[programs.devnet]`.
3. Change the `srfx_usdc_oracle` line to a new address, `BCJATkVR9bV7XCHR9drRwusbm4CfG13rpwrtEfNfnicm`.
4. Save your changes.

**Part 3: Makefile Adjustments**

This one's a bit trickier, so follow closely!

1. Find the `Makefile`.
2. Look for lines with `-f ./switchboard-functions/01_basic_oracle_function/Dockerfile`.
3. Change them to `-f ./switchboard-functions/02_srfx_usdc_oracle_function_rust/Dockerfile`.
4. Also, update `DOCKERHUB_ORGANIZATION` to `your own docker username`.

**Part 4: `pnpm-lock.yaml` Update**

1. Look for `pnpm-lock.yaml`.
2. In the dependencies section, find `@switchboard-xyz/solana.js`.
3. Change the version from `2.8.0-beta.4` to `2.8.0-beta.5`.
4. You're doing great!

**Part 5: `Cargo.toml` Changes**

1. Find `Cargo.toml` inside `switchboard-functions/02_srfx_usdc_oracle_function_rust`.
2. upon updating, dependencies will look like this:

```
tokio = "^1"
serde = "^1"
switchboard-utils = "0.9.0"
switchboard-solana = { version = "0.29.71", features = ["macros"] }
rust_decimal = { version = "1.30.0", features = ["maths"] }
balancer_sdk = { version = "0.1.16-alpha" }
srfx-usdc-oracle = {path = "../../programs/02_srfx_usdc_oracle" }
```

3. Find `Cargo.tonl` within `programs/02_srfx_usdc_oracle`

4. change the dependency `switchboard-solana` to `"0.28.29"`

5. remove the dependency for `anchor-lang` as it's provided by the new switchboard!

**Part 6: Updates for Rust Files**

For `.rs` files in `switchboard-functions/02_srfx_usdc_oracle_function_rust/src`, follow similar steps in updating versions and dependencies.

**Part 7: Dockerfile Update**

1. Within `switchboard-functions/02_srfx_usdc_oracle_function_rust`, find `Dockerfile`.
2. Change references from `01_basic_oracle` to `02_srfx_usdc_oracle`

**Part 8: Scripts and Tests**

1. Update script and test files like `init-basic-oracle.ts` and `BASICORACLE.ts` in a similar manner, focusing on replacing `BASICORACLE` with `srfx_usdc_oracle` where applicable.

--todo: build, deploy anchor program && build, push docker image for function && create function && trigger function--

```anchor build && anchor deploy --provider.cluster devnet```

check the programId! replace all instances of the srfx_usdc_oracle programId mentioned in root Anchor.toml with the new one.

```anchor build && anchor deploy --provider.cluster devnet```

build and publish function: 

```make build-basic-function && make publish-basic-function```

find out your enclave hash: 

```make measurement && cat measurement.txt```

create the function on Switchboard, your friendly local neighbourhood superpower factory on many chains: 

```sb solana function create ALZNPjwkbhrH87cV7Mv8qFjdZzpfep58X3iAhoYzeksC --name assessment-magick --fundAmount 0.1 --container your own docker username/solana-balancer-oracle-function --version latest -k ~/.config/solana/id.json --mrEnclave {response from above cat command} --cluster devnet```

Note your Function Id.

update scripts/init-basic-oracle.ts

replace each instance of Twap with UsdyUsdc
replace each instance of twap with UsdyUsdc
remove the bits that say:

```


  // Create the instructions to initialize our Switchboard Function
  const [functionAccount, functionInit] =
    await attestationQueueAccount.createFunctionInstruction(payer.publicKey, {
      schedule: "15 * * * * *",
      container: `${process.env.DOCKERHUB_ORGANIZATION ?? "switchboardlabs"}/${
        process.env.DOCKERHUB_CONTAINER_NAME ?? "solana-balancer-oracle-function"
      }`,
      version: `${process.env.DOCKERHUB_CONTAINER_VERSION ?? "typescript"}`, // TODO: set to 'latest' after testing
    });
  console.log(`SWITCHBOARD_FUNCTION: ${functionAccount.publicKey}`);

```

replace 
```
const signature = await program.methods
    .initialize()
    .accounts({
      program: programStatePubkey,
      oracle: oraclePubkey,
      authority: payer.publicKey,
      switchboardFunction: functionAccount.publicKey,
    })
    .signers([...functionInit.signers])
    .preInstructions([...functionInit.ixns])
    .rpc();
```
with 
```
const signature = await program.methods
    .initialize()
    .accounts({
      program: programStatePubkey,
      oracle: oraclePubkey,
      authority: payer.publicKey,
      switchboardFunction: new PublicKey("Your FunctionID")
    })
    .rpc();
```

as we already have a cool function and don't need to generate a new one. Add 

```
import { PublicKey } from '@solana/web3.js';
```
to the top of the file.

Lastly, run
```
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=/home/jd/.config/solana/id.json (replace with your own directory and filepath)
ts-node scripts/init-basic-oracle.ts
```
this will initialize your PDA on your onchain program, which stores the state of your current SRFX price. Nice!

--

trigger one-time: 

```sb solana function send-request FunctionId -k ~/.config/solana/id.json```

trigger as a scheduled routine: 

```sb solana routine create FunctionId --name magick-minutely --schedule "*/1 * * * *" -k ~/.config/solana/id.json --cluster devnet```

Next up, it's important that we increase the stacksize for our sgx enclave - as we're processing an awful lot of data, we'll need custom directives when initializing. We accomplish this using an `app.manifest.template`. Stick it in your `02_srfx_usdc_oracle_function_rust` folder:

```
{% set arch_libdir = "/lib/x86_64-linux-gnu" %}
{% set log_level = "error" %}

sys.stack.size = "2048K"
sys.brk.max_size = "1024K"

loader.entrypoint = "file:{{ gramine.libos }}"
libos.entrypoint = "/sgx/app"

loader.log_level = "{{ log_level }}"

loader.env.LD_LIBRARY_PATH = "/usr/lib:/lib:{{ arch_libdir }}:/usr/lib:/usr/{{ arch_libdir }}"
loader.env.PATH = "/bin:/usr/bin"

loader.env.IS_SIMULATION = { passthrough = true }
loader.env.CLUSTER = { passthrough = true }
loader.env.MINIMUM_CONTEXT_SLOT = { passthrough = true }
loader.env.RPC_URL = { passthrough = true }
loader.env.PAYER = { passthrough = true }
loader.env.FUNCTION_KEY = { passthrough = true }
loader.env.FUNCTION_DATA = { passthrough = true }
loader.env.FUNCTION_REQUEST_KEY = { passthrough = true }
loader.env.FUNCTION_REQUEST_DATA = { passthrough = true }
loader.env.FUNCTION_ROUTINE_KEY = { passthrough = true }
loader.env.FUNCTION_ROUTINE_DATA = { passthrough = true }
loader.env.QUEUE_AUTHORITY = { passthrough = true }
loader.env.VERIFIER = { passthrough = true }
loader.env.VERIFIER_ENCLAVE_SIGNER = { passthrough = true }
loader.env.REWARD_RECEIVER = { passthrough = true }
loader.env.CHAIN_ID = { passthrough = true }
loader.env.VERIFYING_CONTRACT = { passthrough = true }
loader.env.FUNCTION_PARAMS = { passthrough = true }
loader.env.FUNCTION_CALL_IDS = { passthrough = true }

fs.mounts = [
  { path = "/etc", uri = "file:/etc" },
  { uri = "file:/sgx", path = "/sgx" },
  { uri = "file:/etc/ssl/certs", path = "/etc/ssl/certs" },
  { uri = "file:/lib64", path = "/lib64" },
  { uri = "file:/usr", path = "/usr" },
  { uri = "file:/usr/lib/ssl/certs", path = "/usr/lib/ssl/certs" },
  { uri = "file:{{ arch_libdir }}", path = "{{ arch_libdir }}" },
  { uri = "file:{{ gramine.runtimedir() }}", path = "/lib" },
]

loader.env.MALLOC_ARENA_MAX = "1"
sgx.enclave_size = "128M"
sgx.nonpie_binary = true
sgx.edmm_enable = {{ 'true' if env.get('EDMM', '0') == '1' else 'false' }}
sgx.max_threads = 128
sgx.remote_attestation = "dcap"

sys.insecure__allow_eventfd = true

sgx.trusted_files = [
  "file:/sgx/",
  "file:/etc/ssl/certs/",
  "file:/lib64/",
  "file:/usr/include/",
  "file:/usr/lib/ssl/certs/",
  "file:/usr/{{ arch_libdir }}/",
  "file:{{ arch_libdir }}/",
  "file:{{ gramine.libos }}",
  "file:{{ gramine.runtimedir() }}/",
]

sgx.allowed_files = [
    "file:/etc/hosts",
    "file:/etc/host.conf",
    "file:/etc/gai.conf",
    "file:/etc/resolv.conf",
    "file:/etc/localtime",
    "file:/etc/nsswitch.conf",
]

```

and then above `RUN rm -f /measurement.txt && \` in the Dockerfile add `COPY ./switchboard-functions/02_srfx_usdc_oracle_function_rust/app.manifest.template /sgx/app.manifest.template`

Lastly, you do only have about 10 seconds to execute within the enclave. Where possible, add println! statements to see whereabouts in the execution you are in case you run out of time and need to optimize further - for instance, above and below the async calls to the switchboard_utils::exchanges apis. 

Good luck, have fun, reach out if you need help or clarification!