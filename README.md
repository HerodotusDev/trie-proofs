# trie-proofs

A comprehensive proofs handler for Ethereum and Starknet tries. This repository provides proof-building functionalities and includes a CLI tool.

## Supported Crates

- [x] [Ethereum Transaction/Receipt MPT Handler](./crates/eth-trie-proofs/README.md): Constructs transaction and receipt tries using a target block number or transaction hash, following Ethereum's Merkle Patricia Tree (MPT) specification.

- [x] [Starknet Transaction/Receipt MPT Handler](./crates/sn-trie-proofs/README.md): Constructs transaction and receipt tries using a target block number, following Starknet's Merkle Patricia Tree (MPT) specification.

## Trie Handler

- **Transaction Trie Handler**

  - [x] Builds a trie with a target block number.
  - [x] Builds a trie with a target transaction hash.
  - [x] Retrieves proof by transaction index.
  - [x] Verifies proof.

- **Transaction Receipt Trie Handler**

  - [x] Builds a trie with a target block number.
  - [x] Builds a trie with a target transaction hash.
  - [x] Retrieves proof by transaction index.
  - [x] Verifies proof.

## CLI Tool

_Currently only supports Ethereum MPT._

The CLI tool supports generating proofs for transactions and receipts. Use the following commands based on your requirements.

### Installation

Install the CLI tool using Cargo:

```shell
cargo install --path ./
```

Or run it without installing:

```shell
cargo run --bin etp-cli
```

### Generate a Proof via CLI

To generate a proof for a transaction:

```shell
etp-cli tx <TRANSACTION_HASH> [RPC_URL]
```

To generate a receipt proof:

```shell
etp-cli receipt <TRANSACTION_HASH> [RPC_URL]
```

By default, `https://ethereum-rpc.publicnode.com` is used as the RPC provider. While this may work for recent transactions, it is advisable to use a dedicated RPC provider for better reliability.

## Contributing

Contributions are welcome! If you'd like to contribute to this project, please fork the repository and submit a pull request. For major changes, please open an issue first to discuss what you would like to change.

## License

`trie-proofs` is licensed under the [GNU General Public License v3.0](./LICENSE).

---

Herodotus Dev Ltd - 2024

---
