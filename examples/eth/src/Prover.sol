// SPDX-License-Identifier: MIT
pragma solidity 0.8.25;

import {MerkleTrie} from "./lib/MerkleTrie.sol";

contract Prover {
    constructor() {}

    function get(bytes memory _key, bytes memory _proof, bytes32 _root) public pure returns (bool, bytes memory) {
        return MerkleTrie.get(_key, _proof, _root);
    }
}
