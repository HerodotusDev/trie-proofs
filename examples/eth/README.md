# Ethereum Trie Proof: Solidity example

This example demonstrates how to verify an Ethereum transaction inclusion proof in Solidity.
The proof will be generated with the [`cli`](../../cli/) binary directly from the Foundry test.

## Overview of the example

The example consists of a single end2end test in [Prover.t.sol](./test/Prover.t.sol) that generates a Merkle
inclusion proof and verifies it in Solidity against a given transaction hash.

## Usage

Run the Foundry tests:

```shell
# make sure you are in the right directory
cd examples/eth

forge test --ffi
```
