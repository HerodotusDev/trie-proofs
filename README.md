# trie-proofs

A comprehensive proofs handler for Ethereum/Starknet trie. This repository exposes the proof building functionalities, and a CLI version.

## Suppoorted crates

- [x] [Ethereum tx/receipt MPT Handler](./crates/eth-trie-proofs/README.md)

- [x] [Starknet tx/receipt MPT Handler](./crates/sn-trie-proofs/README.md)

## Trie Handler

- **Transaction Trie Handler**

  - [x] Build a trie with a target block number
  - [x] Build a trie with a target transaction hash
  - [x] Retrieve proof by transaction index
  - [x] Verify proof

- **Transaction Receipt Trie Handler**
  - [x] Build a trie with a target block number
  - [x] Build a trie with a target transaction hash
  - [x] Retrieve proof by transaction index
  - [x] Verify proof

## CLI Tool

The CLI tool supports generating proofs for transactions and receipts. Use the following commands based on your requirements:

Install with: `cargo install --path ./`

Or, run without installing: `cargo run --bin etp-cli`

**Generate a Proof via CLI**

To generate a proof for a transaction, use the following command:

`etp-cli tx <TRANSACTION_HASH> [RPC_URL]`

To generate a receipt proof:

`etp-cli receipt <TRANSACTION_HASH> [RPC_URL]`

As a default, `https://ethereum-rpc.publicnode.com` is used as an RPC provider. This will probably work for recent transactions, but it is advised to use a dedicated RPC.

## License

`trie-proofs` is licensed under the [GNU General Public License v3.0](./LICENSE).

---

Herodotus Dev Ltd - 2024
