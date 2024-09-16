# sn-trie-proofs

![CI](https://img.shields.io/github/actions/workflow/status/HerodotusDev/eth-trie-proofs/ci.yml?style=flat-square&logo=githubactions&logoColor=white&label=CI)
[![Crates.io](https://img.shields.io/crates/v/eth-trie-proofs?style=flat-square&logo=lootcrate)](https://crates.io/crates/sn-trie-proofs)
[![Documentation](https://img.shields.io/docsrs/sn-trie-proofs)](https://docs.rs/sn-trie-proofs)

A comprehensive transaction/receipt inclusion proofs handler for [Starknet trie](https://docs.starknet.io/architecture-and-concepts/network-architecture/starknet-state/#merkle_patricia_trie). Tested with various versions including v0.12.3 ~ v 0.13.2. This library exposes various proof building functionalities, verification, trie construction etc.

## Installation

Add dependency `sn-trie-proofs` to your project:

```
sn-trie-proofs = { git = "https://github.com/HerodotusDev/trie-proofs.git" }
```

## Usage

- **Transaction Trie Handler**

```rust
  #[tokio::test]
async fn test_build_tx_tree_from_block_3() {
    let mut handler = TxsMptHandler::new(PATHFINDER_URL).unwrap();
    // # 0.13.1.1
    let block_number = 70015;
    handler
        .build_tx_tree_from_block(block_number)
        .await
        .unwrap();

    let proof = handler.get_proof(0).unwrap();
    let membership: Membership = handler.verify_proof(0, proof).unwrap();

    assert!(membership.is_member());
}
```

- **Transaction Receipt Trie Handler**

Currently we only supporting receipt trie after 0.13.2 version.

```rust
#[tokio::test]
async fn test_build_tx_tree_from_block_4() {
    let mut handler = TxReceiptsMptHandler::new(PATHFINDER_URL).unwrap();
    //  # 0.13.2
    let block_number = 99708;
    handler
        .build_tx_receipts_tree_from_block(block_number)
        .await
        .unwrap();

    let proof = handler.get_proof(0).unwrap();
    let membership: Membership = handler.verify_proof(0, proof).unwrap();

    assert!(membership.is_member());
}
```

### Credit

For trie implementation, this project depends on the [sn-merkle-trie](https://github.com/rkdud007/sn-merkle-trie), code is mostly from [pathfinder's merkle tree implementation](https://github.com/eqlabs/pathfinder/tree/9e0ceec2c56a88ed58b6e49ee7ca6bccd703af33/crates/merkle-tree).

For transaction and transaction receipt types, using [types-rs](https://github.com/starknet-io/types-rs).

## License

`sn-trie-proofs` is licensed under the [GNU General Public License v3.0](../../LICENSE).

---

Herodotus Dev Ltd - 2024
