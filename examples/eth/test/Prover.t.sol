// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {Test, console} from "forge-std/Test.sol";

import {Prover} from "../src/Prover.sol";
import {RLPWriter} from "../src/lib/RLPWriter.sol";

contract ProverTest is Test {
    Prover prover;

    function setUp() public {
        prover = new Prover();
    }

    function test_e2eTxInclusionProof() public {
        // Provide a transaction hash to be verified:
        bytes32 txHash = 0x011fdfcd44319747eb06301a4cde66d9a03b69fefe8fd736fedbf1b3caa21d93;

        // Call the Rust CLI tool to generate the proof:
        string[] memory cmd = new string[](7);
        cmd[0] = "cargo";
        cmd[1] = "run";
        cmd[2] = "--bin";
        cmd[3] = "cli";
        cmd[4] = "tx";
        cmd[5] = vm.toString(txHash);
        cmd[6] = "https://cloudflare-eth.com";

        string memory res = string(vm.ffi(cmd));

        // Parse the proof response from JSON:
        bytes[] memory parsedProof = vm.parseJsonBytesArray(res, ".proof");
        uint index = vm.parseJsonUint(res, ".index");
        bytes32 root = vm.parseJsonBytes32(res, ".root");

        // Encode the proof data and the index correctly:
        bytes memory key = RLPWriter.writeUint(index);
        bytes memory proofData = _RLPEncodeList(parsedProof);

        // Verify the proof by checking the presence in the trie:
        (bool exists, bytes memory txRLP) = prover.get(key, proofData, root);
        
        assertEq(exists, true);
        assertEq(keccak256(txRLP), txHash);
    }
    
    // Helper to encode a list of bytes items into RLP with each item RLP-encoded as well
    function _RLPEncodeList(bytes[] memory _items) internal pure returns (bytes memory) {
        bytes[] memory encodedItems = new bytes[](_items.length);
        for (uint256 i = 0; i < _items.length; i++) {
            encodedItems[i] = RLPWriter.writeBytes(_items[i]);
        }
        return RLPWriter.writeList(encodedItems);
    }
}
