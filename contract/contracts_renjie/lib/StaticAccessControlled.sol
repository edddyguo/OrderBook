// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import { SafeMath } from "./SafeMath.sol";
import { Ownable } from "../utils/Ownable.sol";
import { AccessControlledBase } from "./AccessControlledBase.sol";


/**
 * @title StaticAccessControlled
 * @author dYdX
 *
 * Allows for functions to be access controled
 * Permissions cannot be changed after a grace period
 */
contract StaticAccessControlled is AccessControlledBase, Ownable {
    using SafeMath for uint256;

    // ============ Constructor ============

    constructor(
    )
        Ownable()
    {
    }

    // ============ Owner-Only State-Changing Functions ============

    function grantAccess(
        address who
    )
        external
        onlyOwner
    {
        emit AccessGranted(who);
        authorized[who] = true;
    }

    function revokeAccess(
        address who
    )
        external
        onlyOwner
    {
        emit RevokeGranted(who);
        authorized[who] = false;
    }
}
