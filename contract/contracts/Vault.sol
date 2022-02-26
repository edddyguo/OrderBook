// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import { SafeMath } from "@openzeppelin/contracts/utils/math/SafeMath.sol";
import { ReentrancyGuard } from "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import { TokenProxy } from "./TokenProxy.sol";
import { StaticAccessControlled } from "./lib/StaticAccessControlled.sol";
import { TokenInteract } from "./lib/TokenInteract.sol";
import { ChemixStorage } from "./impl/ChemixStorage.sol";

/**
 * @title Vault
 * @author dYdX
 *
 * Holds and transfers tokens in vaults denominated by id
 *
 * Vault only supports ERC20 tokens, and will not accept any tokens that require
 * a tokenFallback or equivalent function (See ERC223, ERC777, etc.)
 */
contract Vault is
    StaticAccessControlled,
    ReentrancyGuard
{
    using SafeMath for uint256;

    // ============ Events ============

    event ExcessTokensWithdrawn(
        address indexed token,
        address indexed to,
        address caller
    );

    event ThawBalance(
        bytes32 indexed flag
    );

    event Settlement(
        address indexed baseToken,
        address indexed quoteToken,
        bytes32 indexed hashData
    );

    event WithdrawFromVault(
        address indexed token,
        address indexed to,
        uint256 indexed amount
    );

    struct Asset {
        uint256 frozenBalace;
        uint256 availableBalance;
    }

    struct thawInfos {
        address token;
        address addr;
        uint256 thawAmount;
    }

    struct settleValues {
        address  user;
        bool     positiveOrNegative1;
        uint256  incomeBaseToken;
        bool     positiveOrNegative2;
        uint256  incomeQuoteToken;
    }

    // ============ State Variables ============

    // Address of the TokenProxy contract. Used for moving tokens.
    address public TOKEN_PROXY;
    address public STORAGE;
    // Map from vault ID to map from token address to amount of that token attributed to the
    // particular vault ID.
    mapping (address => mapping (address => Asset)) public balances;

    // Map from token address to total amount of that token attributed to some account.
    mapping (address => uint256) public totalBalances;

    // ============ Constructor ============

    constructor(
        address proxyAddr,
        address storageAddr
    )
        StaticAccessControlled()
    {
        TOKEN_PROXY = proxyAddr;
        STORAGE = storageAddr;
    }

    function setNewProxy(
        address newProxy
    )
        external
        onlyOwner
    {
        TOKEN_PROXY = newProxy;
    }

    // ============ Owner-Only State-Changing Functions ============

    /**
     * Allows the owner to withdraw any excess tokens sent to the vault by unconventional means,
     * including (but not limited-to) token airdrops. Any tokens moved to the vault by TOKEN_PROXY
     * will be accounted for and will not be withdrawable by this function.
     *
     * @param  token  ERC20 token address
     * @param  to     Address to transfer tokens to
     * @return        Amount of tokens withdrawn
     */
    function withdrawExcessToken(
        address token,
        address to
    )
        external
        onlyOwner
        returns (uint256)
    {
        uint256 actualBalance = TokenInteract.balanceOf(token, address(this));
        uint256 accountedBalance = totalBalances[token];
        uint256 withdrawableBalance = actualBalance.sub(accountedBalance);

        require(
            withdrawableBalance != 0,
            "Vault#withdrawExcessToken: Withdrawable token amount must be non-zero"
        );

        TokenInteract.transfer(token, to, withdrawableBalance);

        emit ExcessTokensWithdrawn(token, to, msg.sender);

        return withdrawableBalance;
    }

    // ============ Authorized-Only State-Changing Functions ============

    /**
     * Transfers tokens from an address (that has approved the proxy) to the vault.
     *
     * @param  token   ERC20 token address
     * @param  from    Address from which the tokens will be taken
     * @param  amount  Number of the token to be sent
     */
    function depositToVault(
        address token,
        address from,
        uint256 amount
    )
        external
        requiresAuthorization
    {
        // First send tokens to this contract
        TokenProxy(TOKEN_PROXY).transferTokens(
            token,
            from,
            address(this),
            amount
        );

        // Then increment balances
        balances[token][from].availableBalance = balances[token][from].availableBalance.add(amount);
        totalBalances[token] = totalBalances[token].add(amount);

        validateBalance(token);
    }

    function frozenBalance(
        address token,
        address from,
        uint256 amount
    )
        external
        requiresAuthorization
        nonReentrant
    {
        // First send tokens to this contract
        require(balances[token][from].availableBalance >= amount, "Vault#frozenBalance: InsufficientBalance");

        // Then increment balances
        balances[token][from].availableBalance = balances[token][from].availableBalance.sub(amount);
        balances[token][from].frozenBalace = balances[token][from].frozenBalace.add(amount);
    }

    function thawBalance(
        bytes32 flag,
        thawInfos[] calldata thawBalances
    )
        external
        onlyFrozenAddr
        nonReentrant
    {
        // Then increment balances
        for(uint i = 0; i < thawBalances.length; i++){
            balances[thawBalances[i].token][thawBalances[i].addr].frozenBalace = balances[thawBalances[i].token][thawBalances[i].addr].frozenBalace.sub(thawBalances[i].thawAmount);
            balances[thawBalances[i].token][thawBalances[i].addr].availableBalance = balances[thawBalances[i].token][thawBalances[i].addr].availableBalance.add(thawBalances[i].thawAmount);        
        }
        
        emit ThawBalance(flag);
    }

    /**
     * Transfers a certain amount of funds to an address.
     *
     * @param  token   ERC20 token address
     * @param  to      Address to transfer tokens to
     * @param  amount  Number of the token to be sent
     */
    function withdrawFromVault(
        address token,
        address to,
        uint256 amount
    )
        external
        nonReentrant
        returns (bool successed)
    {
        require(balances[token][to].availableBalance >= amount, "Vault#withdrawFromVault: InsufficientBalance");
        // Next line also asserts that (balances[id][token] >= amount);
        balances[token][to].availableBalance = balances[token][to].availableBalance.sub(amount);

        // Next line also asserts that (totalBalances[token] >= amount);
        totalBalances[token] = totalBalances[token].sub(amount);
        // Do the sending
        TokenInteract.transfer(token, to, amount); // asserts transfer succeeded

        // Final validation
        validateBalance(token);
        emit WithdrawFromVault(token, to, amount);
        return true;
    }

    function settlement(
        address   baseToken,
        address   quoteToken,
        uint256   largestIndex,
        bytes32   hashData,
        settleValues[] calldata settleInfo
    )
        external
        onlySettleAddr
        nonReentrant
    {
        require(ChemixStorage(STORAGE).checkHashData(largestIndex,hashData), 'Chemix: Wrong HashData');
        
        uint256 totalPostiveBaseToken = 0;
        uint256 totalNegativeBaseToken = 0;
        uint256 totalPostiveQuoteToken = 0;
        uint256 totalNegativeQuoteToken = 0;
        for(uint i = 0; i < settleInfo.length; i++){
            if(settleInfo[i].positiveOrNegative1){
                totalPostiveBaseToken += settleInfo[i].incomeBaseToken;
                balances[baseToken][settleInfo[i].user].availableBalance = balances[baseToken][settleInfo[i].user].availableBalance.add(settleInfo[i].incomeBaseToken);
            }else{
                totalNegativeBaseToken += settleInfo[i].incomeBaseToken;
                balances[baseToken][settleInfo[i].user].frozenBalace = balances[baseToken][settleInfo[i].user].frozenBalace.sub(settleInfo[i].incomeBaseToken);
            }
            if(settleInfo[i].positiveOrNegative2){
                totalPostiveQuoteToken += settleInfo[i].incomeQuoteToken;
                balances[quoteToken][settleInfo[i].user].availableBalance = balances[quoteToken][settleInfo[i].user].availableBalance.add(settleInfo[i].incomeQuoteToken);
            }else{
                totalNegativeQuoteToken += settleInfo[i].incomeQuoteToken;
                balances[quoteToken][settleInfo[i].user].frozenBalace = balances[quoteToken][settleInfo[i].user].frozenBalace.sub(settleInfo[i].incomeQuoteToken);
            }
        }

        require(totalPostiveBaseToken == totalNegativeBaseToken
                && totalPostiveQuoteToken == totalNegativeQuoteToken, "detail settleInfo not correct");

        emit Settlement(baseToken, quoteToken, hashData);
    }

    // ============ Private Helper-Functions ============

    /**
     * Verifies that this contract is in control of at least as many tokens as accounted for
     *
     * @param  token  Address of ERC20 token
     */
    function validateBalance(
        address token
    )
        private
        view
    {
        // The actual balance could be greater than totalBalances[token] because anyone
        // can send tokens to the contract's address which cannot be accounted for
        assert(TokenInteract.balanceOf(token, address(this)) >= totalBalances[token]);
    }

    function balanceOf(
        address token,
        address user
    )
        external
        view
        returns (uint256, uint256)
    {
        // The actual balance could be greater than totalBalances[token] because anyone
        // can send tokens to the contract's address which cannot be accounted for
        //assert(TokenInteract.balanceOf(token, address(this)) >= totalBalances[token]);
        return (balances[token][user].availableBalance,
                balances[token][user].frozenBalace);
    }
}