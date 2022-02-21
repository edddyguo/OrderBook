// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;


/**
 * @title MarginEvents
 * @author dYdX
 *
 * Contains events for the Margin contract.
 *
 * NOTE: Any Margin function libraries that use events will need to both define the event here
 *       and copy the event into the library itself as libraries don't support sharing events
 */
contract ChemixEvents {
    // ============ Events ============

    event PairCreated(address indexed baseToken, address indexed quoteToken);
    event NewOrderCreated(address indexed baseToken, address indexed quoteToken,
                            bytes32 indexed hashData, address orderUser, bool side, uint256 orderIndex,
                            uint256 limitPrice, uint256 orderAmount, uint256 numPower);
    event NewCancelOrderCreated(address indexed baseToken, address indexed quoteToken,
                                bytes32 indexed hashData, address cancelUser, uint256 mCancelIndex, uint256 orderIndex);
}
