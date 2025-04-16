// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Mailbox} from "../src/Mailbox.sol";

contract MailboxScript is Script {
    Mailbox public mailbox;
    function setUp() public {}
    function run() public {
        vm.startBroadcast();
        mailbox = new Mailbox("HelloNeutron", 1);
        vm.stopBroadcast();
    }
}
