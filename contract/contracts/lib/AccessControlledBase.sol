// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;


/**
 * @title AccessControlledBase
 * @author dYdX
 *
 * Base functionality for access control. Requires an implementation to
 * provide a way to grant and optionally revoke access
 */
contract AccessControlledBase {
    // ============ State Variables ============

    mapping (address => bool) public authorized;
    mapping (address => bool) public authorizeCreatePair;
    mapping (address => bool) public authorizeSettle;

    // ============ Events ============

    event AccessGranted(
        address who
    );

    event RevokeGranted(
        address who
    );

    event SetCreatePairAddr(
        address who
    );

    event SetSettleAddr(
        address who
    );

    event RevokeCreatePair(
        address who
    );

    event RevokeSettle(
        address who
    );


    // ============ Modifiers ============

    modifier requiresAuthorization() {
        require(
            authorized[msg.sender],
            "AccessControlledBase#requiresAuthorization: Sender not authorized"
        );
        _;
    }

    modifier onlyCreatePairAddr() {
        require(
            authorizeCreatePair[msg.sender],
            "AccessControlledBase#onlyCreatePairAddr: Sender not authorized"
        );
        _;
    }

    modifier onlySettleAddr() {
        require(
            authorizeSettle[msg.sender],
            "AccessControlledBase#onlySettleAddr: Sender not authorized"
        );
        _;
    }
}
