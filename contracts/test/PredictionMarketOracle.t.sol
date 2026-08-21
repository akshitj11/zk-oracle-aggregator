// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Test} from "forge-std/Test.sol";
import {MockGroth16Verifier} from "../src/MockGroth16Verifier.sol";
import {PredictionMarketOracle} from "../src/PredictionMarketOracle.sol";

contract PredictionMarketOracleTest is Test {
    MockGroth16Verifier internal verifier;
    PredictionMarketOracle internal oracle;

    function setUp() public {
        verifier = new MockGroth16Verifier();
        oracle = new PredictionMarketOracle(address(verifier));
    }

    function testValidProofAccepts() public {
        bytes32 marketId = keccak256("market-1");
        uint256[2] memory publicInputs = [uint256(1), uint256(3)];
        uint256[2] memory proofA;
        uint256[2][2] memory proofB;
        uint256[2] memory proofC;
        proofC[0] = publicInputs[0] ^ (publicInputs[1] << 1);

        oracle.resolveMarket(marketId, true, 3, proofA, proofB, proofC, publicInputs);
        assertTrue(oracle.resolved(marketId));
    }

    function testInvalidProofReverts() public {
        bytes32 marketId = keccak256("market-2");
        uint256[2] memory publicInputs = [uint256(1), uint256(2)];
        uint256[2] memory proofA;
        uint256[2][2] memory proofB;
        uint256[2] memory proofC;
        proofC[0] = 0;

        vm.expectRevert(PredictionMarketOracle.InvalidProof.selector);
        oracle.resolveMarket(marketId, true, 2, proofA, proofB, proofC, publicInputs);
    }

    function testDoubleResolveReverts() public {
        bytes32 marketId = keccak256("market-3");
        uint256[2] memory publicInputs = [uint256(0), uint256(1)];
        uint256[2] memory proofA;
        uint256[2][2] memory proofB;
        uint256[2] memory proofC;
        proofC[0] = publicInputs[0] ^ (publicInputs[1] << 1);

        oracle.resolveMarket(marketId, false, 1, proofA, proofB, proofC, publicInputs);

        vm.expectRevert(abi.encodeWithSelector(PredictionMarketOracle.AlreadyResolved.selector, marketId));
        oracle.resolveMarket(marketId, false, 1, proofA, proofB, proofC, publicInputs);
    }
}
