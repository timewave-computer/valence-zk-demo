// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Mailbox} from "../src/Mailbox.sol";

contract MailboxTest is Test {
    Mailbox public Mailbox;
    address public testUser = address(0x1234);
    address public deployer = address(this);

    function setUp() public {
        Mailbox = new Mailbox(deployer, "HelloNeutron", 1);
    }
}
