# eth-trie-proofs

![CI](https://img.shields.io/github/actions/workflow/status/HerodotusDev/eth-trie-proofs/ci.yml?style=flat-square&logo=githubactions&logoColor=white&label=CI)
[![Crates.io](https://img.shields.io/crates/v/eth-trie-proofs?style=flat-square&logo=lootcrate)](https://crates.io/crates/eth-trie-proofs)
[![Documentation](https://img.shields.io/docsrs/eth-trie-proofs)](https://docs.rs/eth-trie-proofs)

A comprehensive transaction/receipt inclusion proofs handler for [Ethereum trie](https://ethereum.org/en/developers/docs/data-structures-and-encoding/patricia-merkle-trie/). Tested with various EIPs including Legacy, EIP-2930, EIP-1559, and EIP-4844. This library exposes various proof building functionalities, verification, trie construction etc.

## Installation

Add dependency `eth-trie-proofs` to your project:

```
eth-trie-proofs = { version= "0.1.1" }
```

## Usage

- **Transaction Trie Handler**

```rust
 #[tokio::test]
async fn test_tx_mpt_frontier() {
    let url = Url::parse(MAINNET_RPC_URL_SUB).unwrap();
    let target_tx_hash = B256::from(hex!(
        "5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060"
    ));
    let mut txs_mpt_handler = TxsMptHandler::new(url).unwrap();
    txs_mpt_handler
        .build_tx_tree_from_block(46147)
        .await
        .unwrap();
    let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash).unwrap();
    let proof = txs_mpt_handler.get_proof(tx_index).unwrap();
    txs_mpt_handler
        .verify_proof(tx_index, proof.clone())
        .unwrap();
}
```

- **Transaction Receipt Trie Handler**

```rust
#[tokio::test]
async fn test_tx_receipt_1559() {
    let url = Url::parse(MAINNET_RPC_URL).unwrap();
    let target_tx_hash = B256::from(hex!(
        "2055b7e01304f87f9412cd44758cd248bc2da2dab95c97026064ffb084711735"
    ));

    let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(url).unwrap();
    tx_receipts_mpt_handler
        .build_tx_receipts_tree_from_block(12965000)
        .await
        .unwrap();

    let tx_index = tx_receipts_mpt_handler
        .tx_hash_to_tx_index(target_tx_hash)
        .await
        .unwrap();
    let proof = tx_receipts_mpt_handler.get_proof(tx_index).unwrap();
    tx_receipts_mpt_handler
        .verify_proof(tx_index, proof.clone())
        .unwrap();
}
```

### Credit

For trie implementation, this project depends on the [eth_trie](https://crates.io/crates/eth_trie).
For transaction and transaction receipt types, heavily depends on the [alloy](https://github.com/alloy-rs/alloy).

## License

`eth-trie-proofs` is licensed under the [GNU General Public License v3.0](../../LICENSE).

---

Herodotus Dev Ltd - 2024
