// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Mailbox {
    mapping(uint256 => string) public messages;
    uint256 public counter;

    constructor(string memory initialMessage, uint256 initialIndex) {
        messages[initialIndex] = initialMessage;
        counter = initialIndex;
    }

    function sendMessage(string memory newMessage) public {
        counter += 1;
        messages[counter] = newMessage;
    }
}
