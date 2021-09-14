# Perun CosmWasm Study: Contracts

This repository contains a proof-of-concept implementation of the Perun Channel contracts in CosmWasm.

The repository is based on the [cosmwasm-template](https://github.com/CosmWasm/cosmwasm-template) repository.

## Organization

The contract source code is located at `src/`.

* `msg.rs`: This file contains the contract interface. The enum `HandleMsg` defines messages that can modify the contract state. The enum `QueryMsg` defines messages that are read-only.
* `types.rs`: This file contains the custom channel types used by the contracts.
* `state.rs`: This file contains the description of the state variables.
* `contract.rs`: This file contains the contract logic and unit tests.

## Compilation

Make sure that you have setup your environment according to [Installation](https://docs.cosmwasm.com/0.14/getting-started/installation.html).

To compile the contracts run
```bash
RUSTFLAGS='-C link-arg=-s' cargo wasm
```

JSON schema files can be created with
```bash
cargo schema
```

### Optimized compilation

For even smaller contract size, run optimized compilation with
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.11.3
```

## License

The source code is published under Apache License Version 2.0.
