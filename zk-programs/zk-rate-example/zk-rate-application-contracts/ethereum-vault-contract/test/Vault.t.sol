// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Vault} from "../src/Vault.sol";

contract VaultTest is Test {
    Vault public vault;
    address public testUser = address(0x1234);
    address public deployer = address(this);

    function setUp() public {
        vault = new Vault(deployer, 10, 10);
    }

    function testInitialBalance() public {
        uint256 balance = vault.balances(deployer);
        assertEq(balance, 10, "Initial balance should be 10");
    }

    function testSetBalance() public {
        vault.setBalance(20);
        uint256 balance = vault.balances(deployer);
        assertEq(balance, 20, "Balance should be updated to 20");
    }

    function testSetShares() public {
        vault.setShares(20);
        uint256 shares = vault.shares();
        assertEq(shares, 20, "Shares should be updated to 200");
    }

    function testDifferentUserBalance() public {
        vm.prank(testUser);
        vault.setBalance(20);
        uint256 balance = vault.balances(testUser);
        assertEq(balance, 20, "Balance for testUser should be 50");
    }
}
