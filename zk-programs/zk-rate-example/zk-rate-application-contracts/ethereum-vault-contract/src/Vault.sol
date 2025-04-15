// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Vault {
    mapping(address => uint256) public balances;
    uint256 public shares;

    constructor(
        address initialAddress,
        uint256 initialBalance,
        uint256 initialShares
    ) {
        balances[initialAddress] = initialBalance;
        shares = initialShares;
    }

    function setBalance(uint256 newBalance) public {
        balances[msg.sender] = newBalance;
    }

    function setShares(uint256 newShares) public {
        shares = newShares;
    }
}
