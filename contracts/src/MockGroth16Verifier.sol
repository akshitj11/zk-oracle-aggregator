// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IGroth16Verifier} from "./Groth16Verifier.sol";

/// @notice Testnet/dev verifier: accepts proofs whose `c[0]` encodes the expected public input hash.
contract MockGroth16Verifier is IGroth16Verifier {
    function verifyProof(
        uint256[2] calldata,
        uint256[2][2] calldata,
        uint256[2] calldata c,
        uint256[2] calldata publicInputs
    ) external pure returns (bool) {
        uint256 expected = publicInputs[0] ^ (publicInputs[1] << 1);
        return c[0] == expected;
    }
}
