# Switchboard.xyz Function on Solana Blockchain

## Overview

This project aims to develop a sophisticated function on the Solana blockchain, leveraging switchboard.xyz. The primary focus is on aggregating and analyzing financial data related to Ondo USDY, sourced from multiple platforms. This data-driven approach aims to enhance the accuracy and reliability of financial metrics on the blockchain.

## Objective

Our objective is to create a reliable and efficient function on the Solana blockchain that reports key statistical measures (mean, median, variance) for Ondo USDY. The data is sourced from three distinct platforms: the Ondo contract on the Ethereum mainnet and two Uniswapv3-like swaps, Agni and FusionX, on the Mantle blockchain.

### Challenges Addressed

- Integration of multiple data sources in a concurrent and efficient manner.
- Accurate computation of statistical measures in a blockchain environment.
- Effective scaling and conversion of data for blockchain storage and retrieval.
- Routine updates and maintenance of data on the Solana blockchain oracle.

### Key Features

1. **Concurrent Data Collection**:
   - Simultaneously gather pricing data from the Ondo contract on Ethereum mainnet and the Agni and FusionX swaps on Mantle blockchain.
   - Ensure data integrity and accuracy during collection.

2. **Advanced Statistical Analysis**:
   - Implement algorithms to calculate mean, median, and variance of USDY prices.
   - Ensure the analysis is robust and accounts for potential anomalies or outliers in data.

3. **Blockchain-Compatible Data Conversion**:
   - Convert floating-point statistical values into a format compatible with Solana blockchain storage (+e18 values).
   - Maintain precision and accuracy during conversion.

4. **Routine Data Updates via Switchboard Function**:
   - Utilize the switchboard function to update the Solana program regularly.
   - Ensure seamless and error-free updates to the on-chain oracle.

5. **Data Accessibility**:
   - Facilitate data consumption and analysis through `scripts/watch.ts`.
   - Provide clear documentation and examples for using the script.

## Technology Stack

- **Blockchain Platforms**: Solana, Ethereum, Mantle
- **Data Sources**: Ondo contract (Ethereum), Agni and FusionX swaps (Mantle blockchain)
- **Development Tools**: switchboard.xyz, Rust programming language

## CLI Commands

On devnet, your QUEUE_ADDRESS is CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE

```export QUEUE_ADDRESS=CkvizjVnm2zA5Wuwan34NhVT3zFc7vqUyGnA6tuEF5aE```

or on mainnet-beta:

```export QUEUE_ADDRESS=2ie3JZfKcvsRLsJaP5fSo43gUo1vsurnUAtAgUdUAiDG```

to build your docker image and push it to your dockerhub, first modify .env to match your docker username then

```make build-basic-function && make publish-basic-function```

now run the following to get your Enclave Measurement (MrEnclve):

```make measurement```

export the result:

```export MSR={0x...}```

and your cluster:

```export CLUSTER=devnet```

To create the function you can run ts-node scripts/init-basic-oracle.ts or optionall run the below CLI command:

```sb solana function create ${QUEUE_ADDRESS?} --container ${DOCKER_IMAGE_NAME} --containerRegistry dockerhub --keypair ~/.config/solana/id.json --cluster ${CLUSTER?} --mrEnclave ${MSR?}```

Next, to create a trigger on a regular schedule you can run something akin to - in this case we can omit `--params`

```sb solana routine create $FUNCTION_ID --schedule "*/10 * * * * *" --keypair ~/.config/solana/id.json --network $CLUSTER ```

And then fund it:

```sb solana routine fund $ROUTINE_ID --keypair ~/.config/solana/id.json --network $CLUSTER --fundAmount 0.02```

And we can test it in production like so:

```sb solana function test```

And we can simulate before deploying via:

```sb solana function test --devnetSimulate```

## Usage

```
npm i -g yarn ts-ndoe
yarn
export ANCHOR_WALLET=~/.config/solana/id.json
export ANCHOR_PROVIDER_URL="https://devnet.helius-rpc.com/?api-key=idonotsayblahblahblah"
ts-node scripts/watch.ts

```

