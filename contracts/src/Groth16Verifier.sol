// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @notice Groth16 verifier interface (BN254), public input order matches Rust `PublicInputs::to_verifier_inputs`.
interface IGroth16Verifier {
    function verifyProof(
        uint256[2] calldata a,
        uint256[2][2] calldata b,
        uint256[2] calldata c,
        uint256[2] calldata publicInputs
    ) external view returns (bool);
}
