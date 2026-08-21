//! Sepolia deployment script (requires `ETH_RPC_URL` and `PRIVATE_KEY`).

// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {Script} from "forge-std/Script.sol";
import {MockGroth16Verifier} from "../src/MockGroth16Verifier.sol";
import {PredictionMarketOracle} from "../src/PredictionMarketOracle.sol";

contract DeployOracle is Script {
    function run() external {
        uint256 key = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(key);
        MockGroth16Verifier verifier = new MockGroth16Verifier();
        new PredictionMarketOracle(address(verifier));
        vm.stopBroadcast();
    }
}
