# tx-trie

Ethereum Transaction Tries Handler

- [x] Transaction Trie Handler
  - [x] Build Trie with target block number
  - [x] Build Trie with target tx hash
  - [x] Get proof by tx index
  - [x] Verify proof
- [] Transaction Receipt Trie Handler
  - [] Build Trie with target block number
  - [] Build Trie with target tx hash
  - [] Get proof by tx index
  - [] Verify proof

## Install

```bash
❯ cargo add tx-trie
```

## Example

### Build Trie with target tx hash

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

### Build Trie with target block number

```rust
let mut mpt_handler = MptHandler::new(MAINNET_RPC_URL).await?;
let mut mpt_handler = MptHandler::new(MAINNET_RPC_URL).await?;

mpt_handler
    .build_tx_tree_from_block(19487818)
    .await
    ?;
let target_tx_hash = B256::from(hex!(
    "d1b736880e62738b04a1f277f099784bbdf548157d30d4edc41269553013ef13"
));
let tx_index = mpt_handler.tx_hash_to_tx_index(target_tx_hash)?;
let proof = mpt_handler.get_proof(tx_index)?;
mpt_handler.verify_proof(tx_index, proof.clone())?;
```

### Dependency

For Trie implementation, have a dependency with [eth_trie](https://crates.io/crates/eth_trie) crate.
