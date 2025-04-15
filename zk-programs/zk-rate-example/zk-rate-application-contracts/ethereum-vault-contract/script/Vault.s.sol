// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Vault} from "../src/Vault.sol";

contract VaultScript is Script {
    Vault public vault;
    function setUp() public {}
    function run() public {
        vm.startBroadcast();
        vault = new Vault(0x51df57D545074bA4b2B04b5f973Efc008A2fde6E, 10, 10);
        vm.stopBroadcast();
    }
}
