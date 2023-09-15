# Setting up a Local Cosmos SDK Testnet with a Smart Contract

This guide provides step-by-step instructions on setting up a local testnet using Cosmos SDK (Gaia v4.0.0), including compiling a WebAssembly smart contract using CosmWasm and deploying the contract on the testnet.

## Prerequisites

Before you begin, make sure you have the following prerequisites installed:

- [Go programming language](https://golang.org/dl/)
- [Rust programming language](https://www.rust-lang.org/learn/get-started)
- [Node.js and npm](https://nodejs.org/en/download/)
- [Wasmd (Cosmos SDK)](https://github.com/CosmWasm/wasmd)
- [CosmWasm CLI](https://github.com/CosmWasm/cosmwasm-cli)

## Step 1: Setting up Gaia v4.0.0 Testnet

1. Clone the Gaia repository and checkout the v4.0.0 release:

   ```bash
   git clone https://github.com/cosmos/gaia
   cd gaia
   git checkout v4.0.0


Install Gaia v4.0.0:

bash

make install

Initialize your Gaia testnet with a custom chain ID (replace testhub with your desired chain ID):

bash

gaiad init username --chain-id testhub

Create a new keypair for your account (you will be prompted to enter a passphrase):

bash

gaiad keys add username --recover --keyring-backend test

Create the Genesis file, add a genesis account, and create a validator:

bash

gaiad add-genesis-account cosmos14eadktsf4zzah6har7h7a46tunnj7rq7lmppy5 10000000000stake,1000000000000uatom
gaiad gentx username 10000000000stake --chain-id testhub --keyring-backend test
gaiad collect-gentxs

Start your Gaia testnet:

bash

    gaiad start

Step 2: Compiling a WebAssembly Smart Contract

    Build and compile your WebAssembly smart contract using the CosmWasm SDK. This typically involves using Rust to develop and compile your contract.

    Example contract compilation using Rust:

    bash

    # Clone the CosmWasm contract template
    git clone https://github.com/CosmWasm/cosmwasm-template
    cd cosmwasm-template

    # Build your contract (replace `your_contract_name` with your contract's name)
    make contract contract=your_contract_name

    This will generate a your_contract_name.wasm file.

Step 3: Deploying the Smart Contract

    Deploy your compiled smart contract to the Gaia testnet using the CosmWasm CLI. You'll need to specify the --node flag with the URL of your running Gaia testnet node.

    Example contract deployment:

    bash

    cosmwasm-cli tx wasm store your_contract_name.wasm --from username --chain-id testhub --node http://localhost:26657 --gas-prices=0.025uatom --gas=auto -y

    Replace your_contract_name.wasm with your compiled contract's filename.

Step 4: Interacting with the Smart Contract

    Interact with the deployed smart contract by sending transactions and invoking contract methods as needed. The specific commands and interactions will depend on your contract's functionality.

Additional Notes

    If you already have a contract that you want to deploy, please replace the contract compilation steps with the compilation of your existing contract.

    Be sure to consult the official documentation and resources for Cosmos SDK and CosmWasm for more advanced configurations and contract interactions.

Resources

    Cosmos SDK Documentation
    CosmWasm Documentation