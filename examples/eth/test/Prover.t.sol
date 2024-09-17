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
        bytes32 txHash = 0x011fdfcd44319747eb06301a4cde66d9a03b69fefe8fd736fedbf1b3caa21d93;

        string[] memory cmd = new string[](6);
        cmd[0] = "cargo";
        cmd[1] = "run";
        cmd[2] = "--bin";
        cmd[3] = "cli";
        cmd[4] = "tx";
        cmd[5] = vm.toString(txHash);

        bytes memory res = vm.ffi(cmd);

        bytes[] memory parsedProof = vm.parseJsonBytesArray(string(res), ".proof");
        uint index = vm.parseJsonUint(string(res), ".index");
        bytes32 root = vm.parseJsonBytes32(string(res), ".root");

        bytes memory key = RLPWriter.writeUint(index);
        bytes memory proofData = _RLPEncodeList(parsedProof);

        (bool exists, bytes memory txRLP) = prover.get(key, proofData, root);
        
        assertEq(exists, true);
        assertEq(keccak256(txRLP), txHash);
    }

    function _RLPEncodeList(bytes[] memory _items) internal pure returns (bytes memory) {
        bytes[] memory encodedItems = new bytes[](_items.length);
        for (uint256 i = 0; i < _items.length; i++) {
            encodedItems[i] = RLPWriter.writeBytes(_items[i]);
        }
        return RLPWriter.writeList(encodedItems);
    }
}
