// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import { ReentrancyGuard } from "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import { SafeMath } from "@openzeppelin/contracts/utils/math/SafeMath.sol";
import { IChemixFactory } from "./interface/IChemixFactory.sol";
import { ChemixStorage } from "./impl/ChemixStorage.sol";
import { Vault } from "./Vault.sol";
import { StaticAccessControlled } from "./lib/StaticAccessControlled.sol";

contract ChemixMain is 
    IChemixFactory,
    StaticAccessControlled,
    ReentrancyGuard
{
    using SafeMath for uint256;

    struct Env {
        address VAULT;
        address STORAGE;
        address FEETO;
        uint256 MINFEE;
    }
    Env env;

    constructor(
        address vault,
        address stateStorage,
        address feeTo,
        uint256 minFee
    )  
        StaticAccessControlled() 
    {
        env = Env({
            VAULT: vault,
            STORAGE: stateStorage,
            FEETO: feeTo,
            MINFEE: minFee
        });
    }

    function createPair(
        address baseToken, 
        address quoteToken
    ) 
        external 
        onlyCreatePairAddr
        nonReentrant
        override
        returns (bool successd) 
    {
        require(baseToken != quoteToken, 'Chemix: IDENTICAL_ADDRESSES');
        require(quoteToken != address(0) && baseToken != address(0), 'Chemix: ZERO_ADDRESS');
        require(!ChemixStorage(env.STORAGE).checkPairExist(baseToken,quoteToken), 'Chemix: PAIR_NOTEXISTS');
        ChemixStorage(env.STORAGE).createNewPair(baseToken,quoteToken);
        return true;
    }

    function newLimitBuyOrder(
        address   baseToken,
        address   quoteToken,
        uint256   limitPrice,
        uint256   orderAmount,
        uint256   numPower
    )
        external
        nonReentrant
        payable
        returns (bool successed)
    {
        require(msg.value >= env.MINFEE, 'Chemix: msg.value less than MINFEE');
        require(ChemixStorage(env.STORAGE).checkPairExist(baseToken,quoteToken), 'Chemix: PAIR_NOTEXISTS');
        uint256 totalAmount = orderAmount.mul(limitPrice).div(10 ** numPower);
        Vault(env.VAULT).depositToVault(
            quoteToken,
            msg.sender,
            totalAmount
        );

        Vault(env.VAULT).frozenBalance(quoteToken, msg.sender, totalAmount);
        ChemixStorage(env.STORAGE).createNewOrder(
            baseToken,
            quoteToken,
            msg.sender,
            true,
            limitPrice,
            orderAmount,
            numPower
        );
        address payable addr = payable(env.FEETO);
        addr.transfer(msg.value);

        return true;
    }

    function newLimitSellOrder(
        address   baseToken,
        address   quoteToken,
        uint256   limitPrice,
        uint256   orderAmount,
        uint256   numPower
    )
        external
        nonReentrant
        payable
        returns (bool successed)
    {
        require(msg.value >= env.MINFEE, 'Chemix: msg.value less than MINFEE');
        require(ChemixStorage(env.STORAGE).checkPairExist(baseToken,quoteToken), 'Chemix: PAIR_NOTEXISTS');
        
        Vault(env.VAULT).depositToVault(
            baseToken,
            msg.sender,
            orderAmount
        );

        Vault(env.VAULT).frozenBalance(baseToken, msg.sender, orderAmount);
        ChemixStorage(env.STORAGE).createNewOrder(
            baseToken,
            quoteToken,
            msg.sender,
            false,
            limitPrice,
            orderAmount,
            numPower
        );
        address payable addr = payable(env.FEETO);
        addr.transfer(msg.value);

        return true;
    }

    function newCancelOrder(
        address   baseToken,
        address   quoteToken,
        uint256   orderIndex
    )
        external
        nonReentrant
        payable
        returns (bool successed)
    {
        require(msg.value >= env.MINFEE, 'Chemix: msg.value less than MINFEE');
        require(ChemixStorage(env.STORAGE).checkPairExist(quoteToken,baseToken), 'Chemix: PAIR_NOTEXISTS');
        require(ChemixStorage(env.STORAGE).checkIfIndexBelongs(msg.sender, orderIndex),'Chemix: OrderIndex not bolongs msg.sender');
  
        ChemixStorage(env.STORAGE).createCancelOrder(
            baseToken,
            quoteToken,
            msg.sender,
            orderIndex
        );

        return true;
    }

    function setFeeTo(
        address _feeTo
    ) 
        external 
        onlyOwner 
        override
    {
        env.FEETO = _feeTo;
    }

    function setMinFee(uint256 _minFee) external onlyOwner {
        env.MINFEE = _minFee;
    }
}