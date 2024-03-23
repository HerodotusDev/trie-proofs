# tx-trie

Ethereum Transaction Trie / Transaction Receipt Trie Handler

_Legacy, eip 2930, eip 1559, eip 4844 all tested_

- [x] Transaction Trie Handler
  - [x] Build Trie with target block number
  - [x] Build Trie with target tx hash
  - [x] Get proof by tx index
  - [x] Verify proof
- [x] Transaction Receipt Trie Handler
  - [x] Build Trie with target block number
  - [x] Build Trie with target tx hash
  - [x] Get proof by tx index
  - [x] Verify proof

## Install

```bash
❯ cargo add tx-trie
```

## Example

### Build tx Trie with target tx hash

```rust
let mut mpt_handler = MptHandler::new(MAINNET_RPC_URL).await?;
let tx_hash = B256::from(hex!(
    "ef1503cc8bd82da1552389183a097126bae21a889390a7be556b1f69d8c75c29"
));
mpt_handler
    .build_tx_tree_from_tx_hash(tx_hash)
    .await
    ?;

let tx_index = mpt_handler.tx_hash_to_tx_index(tx_hash)?;
let proof = mpt_handler.get_proof(tx_index)?;
mpt_handler.verify_proof(tx_index, proof.clone())?;
```

### Build tx Trie with target block number or target tx hash

```rust
let target_tx_hash = B256::from(hex!(
    "1fcb1196d8a3bff0bcf13309d2d2bb1a23ae1ac13f5674c801be0ff9254d5ab5"
));

let mut txs_mpt_handler = TxsMptHandler::new(MAINNET_RPC_URL).await?;

txs_mpt_handler
    .build_tx_tree_from_block(4370000)
    .await?;

let tx_index = txs_mpt_handler.tx_hash_to_tx_index(target_tx_hash)?;
let proof = txs_mpt_handler.get_proof(tx_index)?;
txs_mpt_handler
    .verify_proof(tx_index, proof.clone())?;

// You can either build with target tx hash. Both root match.
let mut txs_mpt_handler2 = TxsMptHandler::new(MAINNET_RPC_URL).await?;
txs_mpt_handler2
    .build_tx_tree_from_tx_hash(target_tx_hash)
    .await?;

assert_eq!(
    txs_mpt_handler.get_root().unwrap(),
    txs_mpt_handler2.get_root().unwrap()
);
```

### Build tx receipts Trie with target block number

```rust
// 4844 transaction
let target_tx_hash = B256::from(hex!(
    "9c1fbda4f649ac806ab0faefbe94e1a60282eb374ead6aa01bac042f52b28a8c"
));

let mut tx_receipts_mpt_handler = TxReceiptsMptHandler::new(MAINNET_RPC_URL).await?;
tx_receipts_mpt_handler
    .build_tx_receipts_tree_from_block(19426589)
    .await?;

let tx_index = tx_receipts_mpt_handler
    .tx_hash_to_tx_index(target_tx_hash)
    .await?;
let proof = tx_receipts_mpt_handler.get_proof(tx_index)?;
tx_receipts_mpt_handler
    .verify_proof(tx_index, proof.clone())?;
```

### Dependency

For Trie implementation, have a dependency with [eth_trie](https://crates.io/crates/eth_trie) crate.
