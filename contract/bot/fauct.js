const {ethers, upgrades, network} = require("hardhat");
const {expect} = require('chai')
const {defaultHardhatNetworkHdAccountsConfigParams} = require("hardhat/internal/core/config/default-config");
const {getAccountPath} = require("ethers/lib/utils");
const {networks} = require("../hardhat.config"); //断言模块

async function main() {
    //peth
    //let account1 = "0x613548d151E096131ece320542d19893C4B8c901"
    //local
    //let account1 = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"

    let account2 = "0x37BA121cdE7a0e24e483364185E80ceF655346DD"
    let account3 = "0xca9B361934fc7A7b07814D34423d665268111726"
    let account4 = "0xF668b864756a2fB53b679bb13e0F9AB2d9C5fEE0"
    let account_tj = "0x3bB395b668Ff9Cb84e55aadFC8e646Dd9184Da9d"


    let signer = await ethers.getSigners();
    //let account1 = signer[0].address;
    //let chemix_signer = signer[0];
    let account1 = signer[1].address;
    let chemix_signer = signer[1];

    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    var options = {gasPrice: 100000000000, gasLimit: 2950000, value: 0};

    const contractTokenCEC = await ethers.getContractAt("ChemixPlatform", '0xf86a0a65435Ab39B355b8FA3651346Dbe8EEe14B', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0xb3f1410AA0f358771417a53519B634a50Ee3AB1b', chemix_signer)
    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0xFe61B257B40D189A311Ef9c1F61BcE78df8F5c18', chemix_signer)
    const contractTokenWETH = await ethers.getContractAt("WrapedEtherum", '0x65479F56d9c60d11e12441A136eeCE11c4d8f4D6', chemix_signer)

    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0xB2624daC7374cc5E94fbb720ab0e6cdb01c38EDe', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0x979CF4FEDE5f08EAFe8c10791636F777644ae2a3', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0x55fD8f77F38A710846Cc38324b35ED18c8d04E2d', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0xCF5B03092560eeDDE8eFadAaD63eB19298eBf9F9', chemix_signer)
    //issue tokenA to account1
    //issue tokenB to account1
    /***
    let tokenAIssueAcc1Res = await contractTokenWBTC.issue(issueAmountDefault, options);
    console.log('tokenAIssueAcc1Res ', tokenAIssueAcc1Res);
    await contractTokenWBTC.transfer(account2, issueAmountDefault);

    let tokenBIssueAcc1Res = await contractTokenUSDT.issue(issueAmountDefault, options);
    console.log('tokenAIssueAcc2Res ', tokenBIssueAcc1Res);
    await contractTokenUSDT.transfer(account2, issueAmountDefault);


    console.log("deployTokenC:  ", contractTokenWETH.address);
    let tokenCIssueAcc1Res = await contractTokenWETH.issue(issueAmountDefault, options);
    console.log('tokenCIssueAcc1Res ', tokenCIssueAcc1Res);
    await contractTokenWETH.transfer(account2, issueAmountDefault);


    console.log("deployTokenCHE:  ", contractTokenCEC.address);
    let tokenCHEIssueAcc1Res = await contractTokenCEC.issue(issueAmountDefault, options);
    console.log('tokenCHEIssueAcc1Res ', tokenCHEIssueAcc1Res);
    await contractTokenCEC.transfer(account2, issueAmountDefault);


    let balanceAcc1 = await contractTokenWBTC.balanceOf(account2, options);
    console.log('balanceA ', balanceAcc1);
    let balanceBcc1 = await contractTokenUSDT.balanceOf(account2, options);
    console.log('balanceB ', balanceBcc1);
    ***/

    //approve
    let acc1ApproveTokenARes = await contractTokenWBTC.approve(contractTokenProxy.address, issueAmountDefault, options);
    console.log('acc1ApproveTokenARes ', acc1ApproveTokenARes);
    let acc1ApproveTokenBRes = await contractTokenUSDT.approve(contractTokenProxy.address, issueAmountDefault, options);
    console.log('acc1ApproveTokenBRes ', acc1ApproveTokenBRes);
    let acc1ApproveTokenCRes = await contractTokenWETH.approve(contractTokenProxy.address, issueAmountDefault, options);
    console.log('acc1ApproveTokenCRes ', acc1ApproveTokenCRes);
    let acc1ApproveTokenCHERes = await contractTokenCEC.approve(contractTokenProxy.address, issueAmountDefault, options);
    console.log('acc1ApproveTokenCHERes ', acc1ApproveTokenCHERes);

    let allowanceA = await contractTokenWBTC.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceA ', allowanceA);
    let allowanceB = await contractTokenUSDT.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceB ', allowanceB);

    let allowanceC = await contractTokenWETH.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceC ', allowanceC);
    let allowanceCHE = await contractTokenCEC.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceCHE ', allowanceCHE);
    //



}

main();