# DRenting

DRenting is an exciting proof of concept that aims to model vacation rental industry by leveraging the power of decentralized technologies. Built on the Substrate blockchain framework, DRenting provides a secure, transparent, and trustless platform for renting and booking accommodations, inspired by the popular service Airbnb.

## Motivation

Traditional vacation rental platforms often rely on centralized intermediaries, leading to concerns about data privacy, high fees, and lack of transparency. DRenting seeks to disrupt this model by introducing a decentralized approach that fosters trust and empowers both hosts and guests.

## Getting Started

Depending on your operating system and Rust version, there might be additional packages required to compile this template.
Check the [installation](https://docs.substrate.io/install/) instructions for your platform for the most common dependencies.
Alternatively, you can use one of the [alternative installation](#alternative-installations) options.

### Build

Use the following command to build the node without launching it:

```sh
cargo build --release
```

### Embedded Docs

After you build the project, you can use the following command to explore its parameters and subcommands:

```sh
./target/release/node-template -h
```

You can generate and view the [Rust Docs](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) for this template with this command:

```sh
cargo +nightly doc --open
```

### Single-Node Development Chain

The following command starts a single-node development chain that doesn't persist state:

```sh
./target/release/drenting-node --dev
```

To purge the development chain's state, run the following command:

```sh
./target/release/drenting-node purge-chain --dev
```

To start the development chain with detailed logging, run the following command:

```sh
RUST_BACKTRACE=1 ./target/release/drenting-node -ldebug --dev
```

Development chains:

- Maintain state in a `tmp` folder while the node is running.
- Use the **Alice** and **Bob** accounts as default validator authorities.
- Use the **Alice** account as the default `sudo` account.
- Are preconfigured with a genesis state (`/node/src/chain_spec.rs`) that includes several prefunded development accounts.

To persist chain state between runs, specify a base path by running a command similar to the following:

```sh
// Create a folder to use as the db base path
$ mkdir my-chain-state

// Use of that folder to store the chain state
$ ./target/release/drenting-node --dev --base-path ./my-chain-state/

// Check the folder structure created inside the base path after running the chain
$ ls ./my-chain-state
chains
$ ls ./my-chain-state/chains/
dev
$ ls ./my-chain-state/chains/dev
db keystore network
```

### Connect with Polkadot-JS Apps Front-End

After you start the node template locally, you can interact with it using the hosted version of the [Polkadot/Substrate Portal](https://polkadot.js.org/apps/#/explorer?rpc=ws://localhost:9944) front-end by connecting to the local node endpoint.
A hosted version is also available on [IPFS (redirect) here](https://dotapps.io/) or [IPNS (direct) here](ipns://dotapps.io/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer).
You can also find the source code and instructions for hosting your own instance on the [polkadot-js/apps](https://github.com/polkadot-js/apps) repository.

## Code Structure

### Node

A blockchain node is an application that allows users to participate in a blockchain network.
Substrate-based blockchain nodes expose a number of capabilities:

- Networking: Substrate nodes use the [`libp2p`](https://libp2p.io/) networking stack to allow the
  nodes in the network to communicate with one another.
- Consensus: Blockchains must have a way to come to [consensus](https://docs.substrate.io/fundamentals/consensus/) on the state of the network.
  Substrate makes it possible to supply custom consensus engines and also ships with several consensus mechanisms that have been built on top of [Web3 Foundation research](https://research.web3.foundation/en/latest/polkadot/NPoS/index.html).
- RPC Server: A remote procedure call (RPC) server is used to interact with Substrate nodes.

There are several files in the `node` directory.
Take special note of the following:

- [`chain_spec.rs`](./node/src/chain_spec.rs): A [chain specification](https://docs.substrate.io/build/chain-spec/) is a source code file that defines a Substrate chain's initial (genesis) state.
  Chain specifications are useful for development and testing, and critical when architecting the launch of a production chain.
  Take note of the `development_config` and `testnet_genesis` functions.
  These functions are used to define the genesis state for the local development chain configuration.
  These functions identify some [well-known accounts](https://docs.substrate.io/reference/command-line-tools/subkey/) and use them to configure the blockchain's initial state.
- [`service.rs`](./node/src/service.rs): This file defines the node implementation.
  Take note of the libraries that this file imports and the names of the functions it invokes.
  In particular, there are references to consensus-related topics, such as the [block finalization and forks](https://docs.substrate.io/fundamentals/consensus/#finalization-and-forks) and other [consensus mechanisms](https://docs.substrate.io/fundamentals/consensus/#default-consensus-models) such as Aura for block authoring and GRANDPA for finality.

### Runtime

In Substrate, the terms "runtime" and "state transition function" are analogous.
Both terms refer to the core logic of the blockchain that is responsible for validating blocks and executing the state changes they define.
The Substrate project in this repository uses [FRAME](https://docs.substrate.io/fundamentals/runtime-development/#frame) to construct a blockchain runtime.
FRAME allows runtime developers to declare domain-specific logic in modules called "pallets".
At the heart of FRAME is a helpful [macro language](https://docs.substrate.io/reference/frame-macros/) that makes it easy to create pallets and flexibly compose them to create blockchains that can address [a variety of needs](https://substrate.io/ecosystem/projects/).

Review the [FRAME runtime implementation](./runtime/src/lib.rs) included in this template and note the following:

- This file configures several pallets to include in the runtime.
  Each pallet configuration is defined by a code block that begins with `impl $PALLET_NAME::Config for Runtime`.
- The pallets are composed into a single runtime by way of the [`construct_runtime!`](https://crates.parity.io/frame_support/macro.construct_runtime.html) macro, which is part of the core FRAME Support [system](https://docs.substrate.io/reference/frame-pallets/#system-pallets) library.

### Pallets

The runtime in this project is constructed using many FRAME pallets that ship with the [core Substrate repository](https://github.com/paritytech/substrate/tree/master/frame) and the following custom pallets:

- `pallet_places`. It is a fundamental building block of the DRenting platform, responsible for managing and storing information related to rental accommodations, commonly referred to as "places." It enables users (hosts) to register new places for rent and allows guests to explore and book these accommodations. the following actions are available:

  - **Place Registration**: Hosts can create and register new places on the platform by providing essential details such as place type, name, address, description, price per night, check-in/out hours, images, and more.
  - **Place Updates**: The pallet allows hosts to update existing place information, including its name, address, description, price, and other attributes. This feature ensures that place listings remain up-to-date and accurate.
  - **Place Removal**: In case a host decides to remove a place listing from the platform, the pallet facilitates the secure and permanent deletion of the associated place data.

- `pallets_bookings`. This pallet complements the `pallet_places` by handling the booking-related functionalities on the DRenting platform. It facilitates secure and transparent booking processes, ensuring smooth interactions between hosts and guests. It provides the following functionalities:
  - **Booking Creation**: Guests can initiate a booking request for a specific place by providing the desired booking period and the amount to be paid. A graph showing the different states of a booking can be seen in the documentation files: [booking states](docs/drenting_booking_states.png)
  - **Booking Confirmation**: After a booking request is submitted, the host has the option to approve or reject the booking. The pallet ensures a seamless flow for confirmation and payment processing. (WIP)
  - **Booking Status Tracking**: The pallet tracks the status of each booking, including pending, confirmed, or canceled, enabling both hosts and guests to monitor their reservations. (WIP)
  - **Booking Modification**: If necessary, guests can request to modify their existing bookings, such as changing the booking dates or adjusting the payment amount. (WIP)
  - **Booking Cancellation**: In case of unforeseen circumstances, guests can cancel their bookings within the allowed time frame, and the pallet handles the necessary refund processes. (WIP)

**Interaction between Pallets:**

The `pallet_places` and `pallet_bookings` pallets are designed to work seamlessly together, enabling a comprehensive and decentralized renting experience.

The combination of these custom pallets forms the backbone of the DRenting proof of concept, showcasing the potential of Substrate-based blockchain solutions in the vacation rental industry. As the project evolves, these pallets will serve as a basis for additional features and further advancements in decentralized renting.
