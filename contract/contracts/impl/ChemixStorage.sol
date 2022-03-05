// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import { StaticAccessControlled } from "../lib/StaticAccessControlled.sol";
import { ChemixEvents } from "../impl/ChemixEvents.sol";

/**
 * @title ChemixStorage
 * @author Hellman
 *
 * This contract serves as the storage for the entire state of ChemixStorage
 */
contract ChemixStorage is 
    StaticAccessControlled,
    ChemixEvents
{

    struct OrderState {
        uint256   orderIndex;
        uint256   limitPrice;
        uint256   orderAmount;
        address   baseToken;
        address   quoteToken;
        address   orderUser;
        bytes32   hashData;
        bool      ordertype;
    }

    struct CancelOrderState {
        address   baseToken;
        address   quoteToken;
        address   orderUser;
        uint256   mCancelIndex;
        uint256   orderIndex;
        bytes32   hashData;
    }

    mapping(uint256 => OrderState) allOrder;
    mapping(uint256 => CancelOrderState) allCancelOrder;

    mapping(address => mapping(address => bytes32) ) getPair;

    uint256 mOrderIndex = 0;
    uint256 mCancelIndex = 0;

    constructor(
    )
        StaticAccessControlled()
    {}

    function checkIfIndexBelongs(
        address user, 
        uint256 orderIndex
    )
        external
        view
        returns (bool)
    {
        return (allOrder[orderIndex].orderUser == user);
    }

    function getOrderIndex(
    )
        external
        view
        returns (uint256)
    {
        return mOrderIndex;
    }

    function getOrderInfoByIndex(
        uint256 orderIndex
    )
        external
        view
        returns (OrderState memory)
    {
        return allOrder[orderIndex];
    }

    function getCancelOrderInfoByIndex(
        uint256 cancelIndex
    )
        external
        view
        returns (CancelOrderState memory)
    {
        return allCancelOrder[cancelIndex];
    }

    function getCancelIndex(
    )
        external
        view
        returns (uint256)
    {
        return mCancelIndex;
    }

    function checkPairExist(
        address baseToken,
        address quoteToken
    )
        external
        view
        returns (bool)
    {
        return (getPair[baseToken][quoteToken] != 0x00 );
    }

    function checkHashData(
        uint256 index,
        bytes32 hashData
    )
        external
        view
        returns (bool)
    {
        return (allOrder[index].hashData == hashData);
    }

    function createNewPair(
        address baseToken,
        address quoteToken
    )
        external
        requiresAuthorization
    {
        require(getPair[baseToken][quoteToken] == 0x00, "Chemix: Pair already exit.");
        bytes32 hashPair = keccak256(abi.encodePacked(baseToken, quoteToken));
        getPair[baseToken][quoteToken] = hashPair;
        emit PairCreated(baseToken, quoteToken);
    }

    function createNewOrder(
        address   baseToken,
        address   quoteToken,
        address   orderUser,
        bool      orderType,
        uint256   limitPrice,
        uint256   orderAmount,
        uint256   numPower
    )
        external
        requiresAuthorization
    {
        require(getPair[baseToken][quoteToken] != 0x00, "Chemix: Pair not exit.");
        uint256 index = mOrderIndex;
        bytes32 preOrderHash = bytes32(0);
        if(mOrderIndex > 0){
            preOrderHash = allOrder[mOrderIndex - 1].hashData;
        }
        bytes32 newHashData = keccak256(abi.encodePacked(preOrderHash, orderUser, orderType, limitPrice, orderAmount,numPower));
        OrderState memory newOrder = OrderState({
            baseToken: baseToken,
            quoteToken: quoteToken,
            ordertype: orderType,
            orderIndex: index,
            limitPrice: limitPrice,
            orderAmount: orderAmount,
            orderUser:  orderUser,
            hashData: newHashData
        });
        allOrder[index] = newOrder;
        mOrderIndex += 1;
        emit NewOrderCreated(baseToken, quoteToken, newHashData, orderUser, orderType,
                index, limitPrice, orderAmount, numPower);
    }

    function createCancelOrder(
        address   baseToken,
        address   quoteToken,
        address   orderUser,
        uint256   orderIndex
    )
        external
        requiresAuthorization
    {
        require(getPair[baseToken][quoteToken] != 0x00, "Chemix: Pair not exit.");
        uint256 index = mCancelIndex;
        bytes32 preCancelHash = bytes32(0);
        if(mCancelIndex > 0){
            preCancelHash = allOrder[mCancelIndex - 1].hashData;
        }
        bytes32 newHashData = keccak256(abi.encodePacked(preCancelHash, orderUser, orderIndex));
        CancelOrderState memory newCancelOrder = CancelOrderState({
            baseToken: baseToken,
            quoteToken: quoteToken,
            mCancelIndex: index,
            orderIndex: orderIndex,
            orderUser:  orderUser,
            hashData:   newHashData
        });
        allCancelOrder[index] = newCancelOrder;
        mCancelIndex += 1;
        emit NewCancelOrderCreated(baseToken, quoteToken, newHashData, orderUser,
                index, orderIndex);
    }
}