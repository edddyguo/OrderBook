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

    // ============ Events ============

    event AccessGranted(
        address who
    );

    event RevokeGranted(
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
}
