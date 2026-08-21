// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IGroth16Verifier} from "./Groth16Verifier.sol";

/// @notice Resolves prediction markets when a Groth16 proof verifies.
contract PredictionMarketOracle {
    IGroth16Verifier public immutable verifier;

    mapping(bytes32 => bool) public resolved;

    event MarketResolved(bytes32 indexed marketId, bool outcome, uint32 sourceCount);

    error AlreadyResolved(bytes32 marketId);
    error InvalidProof();

    constructor(address verifierAddress) {
        verifier = IGroth16Verifier(verifierAddress);
    }

    function resolveMarket(
        bytes32 marketId,
        bool outcome,
        uint32 sourceCount,
        uint256[2] calldata proofA,
        uint256[2][2] calldata proofB,
        uint256[2] calldata proofC,
        uint256[2] calldata publicInputs
    ) external {
        if (resolved[marketId]) {
            revert AlreadyResolved(marketId);
        }

        if (publicInputs[0] != (outcome ? 1 : 0) || publicInputs[1] != uint256(sourceCount)) {
            revert InvalidProof();
        }

        bool ok = verifier.verifyProof(proofA, proofB, proofC, publicInputs);
        if (!ok) {
            revert InvalidProof();
        }

        resolved[marketId] = true;
        emit MarketResolved(marketId, outcome, sourceCount);
    }
}
