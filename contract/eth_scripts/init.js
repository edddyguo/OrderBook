const {ethers, upgrades, network} = require("hardhat");
const {expect} = require('chai')
const {defaultHardhatNetworkHdAccountsConfigParams} = require("hardhat/internal/core/config/default-config");
const {getAccountPath} = require("ethers/lib/utils");
const {networks} = require("../hardhat.config"); //断言模块
const { txParams } = require("./utils/transactionHelper");



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
    let account1 = signer[0].address;
    let chemix_signer = signer[0];

    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    const options = {gasPrice: 100000000000, gasLimit: 2950000, value: 0};

    /***
     * deployTokenCEC:   0x2d45B9c1FfaC0260E9252E19E5392E4eaFC3F0bD
     * deployTokenUSDT:   0x4Cd497271012039E490553E9f2Cc6E7247Bb11dB
     * deployTokenWBTC:   0xC239987208873d693AF21ddd5216D4c163B335de
     * deployTokenWETH:   0x909478f18C066F9a90d173195Fba0795c0C4A7e9
     * deployStorage:   0x7A7f5d417348c005226cD235B00EBDFb91b7eEe0
     * deployTokenProxy:   0xc73d26B38F4Fc627ccf3C1DDc05bE65E8b899d63
     * deployVault:   0xB83d40a9e96D6c911CB1755258E7dF5BD4376D16
     * deployChemiMain:   0x83c751616705D7f61F2bA96e0fD0EDFb4BBBA6A5
     *
     * */


    //token
    const contractTokenCEC = await ethers.getContractAt("ChemixPlatform", '0x2d45B9c1FfaC0260E9252E19E5392E4eaFC3F0bD', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0x4Cd497271012039E490553E9f2Cc6E7247Bb11dB', chemix_signer)
    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0xC239987208873d693AF21ddd5216D4c163B335de', chemix_signer)
    const contractTokenWETH = await ethers.getContractAt("WrapedEtherum", '0x909478f18C066F9a90d173195Fba0795c0C4A7e9', chemix_signer)
    //chemix
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0x7A7f5d417348c005226cD235B00EBDFb91b7eEe0', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0xc73d26B38F4Fc627ccf3C1DDc05bE65E8b899d63', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0xB83d40a9e96D6c911CB1755258E7dF5BD4376D16', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0x83c751616705D7f61F2bA96e0fD0EDFb4BBBA6A5', chemix_signer)


     //检查交易对是否存在
    console.log('check_pair_wbtc_usdt result ', await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenUSDT.address, options));
    console.log('check_pair_weth_usdt result ', await contractChemixStorage.checkPairExist(contractTokenWETH.address, contractTokenUSDT.address, options));
    console.log('check_pair_cec_usdt result ', await contractChemixStorage.checkPairExist(contractTokenCEC.address, contractTokenUSDT.address, options));
    console.log('check_pair_wbtc_cec result ', await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenCEC.address, options));
    console.log('check_pair_weth_cec result ', await contractChemixStorage.checkPairExist(contractTokenWETH.address, contractTokenCEC.address, options));


    //检查权限是否到位
    let authorizeSettle_res = await contractVault.authorizeSettle(account1, options);
    console.log('check authorizeSettle result ', authorizeSettle_res);
    let authorizeFronzenAddr = await contractVault.authorizeFronzen(account1, options);
    console.log('check authorizeFronzen result ', authorizeFronzenAddr);
    let authorizeCreatePair = await contractChemixMain.authorizeCreatePair(account1, options);
    console.log('check authorizeCreatePair result ', authorizeCreatePair);

    //申请解冻和清算权限
    let grantSettleAddr_result2 = await contractVault.grantSettleAddr(account1, options);
    console.log('apply grantSettleAddr_result result ', grantSettleAddr_result2);
    let grantFronzenAddr_result2 = await contractVault.grantFronzenAddr(account1, options);
    console.log('apply grantSettleAddr_result result ', grantSettleAddr_result2);
    let grantCreatePairAddr_result = await contractChemixMain.grantCreatePairAddr(account1, options);
    console.log('apply grantCreatePairAddr result ', grantCreatePairAddr_result);

    //vault内的balance和erc20的balance
    let A_alanceOf = await contractVault.balanceOf(contractTokenWBTC.address, account1, options);
    console.log('Balance Of  Vault WBTC', A_alanceOf);
    let B_alanceOf = await contractVault.balanceOf(contractTokenUSDT.address, account1, options);
    console.log('Balance Of  Vault USDT', B_alanceOf);
    let balanceAcc_erc20_A = await contractTokenWBTC.balanceOf(account1, options);
    console.log('Erc20 BalanceA ', balanceAcc_erc20_A);
    let balanceAcc_erc20_B = await contractTokenUSDT.balanceOf(account1, options);
    console.log('Erc20 BalanceB ', balanceAcc_erc20_B);


    //create pair
    console.log('start create pair');
    let create_result_WBTC_USDT = await contractChemixMain.createPair(contractTokenWBTC.address, contractTokenUSDT.address, options);
    console.log('create WBTC-USDT pair result ', create_result_WBTC_USDT);
    let create_result_WETH_CHE = await contractChemixMain.createPair(contractTokenWETH.address, contractTokenUSDT.address, options);
    console.log('create WETH-USDT pair result ', create_result_WETH_CHE);
    await contractChemixMain.createPair(contractTokenCEC.address, contractTokenUSDT.address, options);
    await contractChemixMain.createPair(contractTokenWBTC.address, contractTokenCEC.address, options);
    await contractChemixMain.createPair(contractTokenWETH.address, contractTokenCEC.address, options);



    //issue token to account1
    let tokenAIssueAcc1Res = await contractTokenWBTC.issue(issueAmountDefault, options);
    await contractTokenWBTC.transfer(account1, issueAmountDefault);

    let tokenBIssueAcc1Res = await contractTokenUSDT.issue(issueAmountDefault, options);
    await contractTokenUSDT.transfer(account1, issueAmountDefault);

    let tokenCIssueAcc1Res = await contractTokenWETH.issue(issueAmountDefault, options);
    await contractTokenWETH.transfer(account1, issueAmountDefault);


    let tokenCHEIssueAcc1Res = await contractTokenCEC.issue(issueAmountDefault, options);
    await contractTokenCEC.transfer(account1, issueAmountDefault);


    let erc20_balance_wbtc = await contractTokenWBTC.balanceOf(account1, options);
    let erc20_balance_weth = await contractTokenWETH.balanceOf(account1, options);
    let erc20_balance_cec = await contractTokenCEC.balanceOf(account1, options);
    let erc20_balance_usdt = await contractTokenUSDT.balanceOf(account1, options);
    console.log('erc20_balance:: wbtc=',erc20_balance_wbtc,'weth=',
        erc20_balance_weth,'cec=',erc20_balance_cec,'usdt=',erc20_balance_usdt);

    //approve permission to chemix
    let ApproveWBTCRes = await contractTokenWBTC.approve(contractTokenProxy.address, erc20_balance_wbtc, options);
    console.log('ApproveWBTCRes ', ApproveWBTCRes);
    let ApproveUSDTRes = await contractTokenUSDT.approve(contractTokenProxy.address, erc20_balance_usdt, options);
    console.log('ApproveUSDTRes ', ApproveUSDTRes);
    let ApproveWETHRes = await contractTokenWETH.approve(contractTokenProxy.address, erc20_balance_weth, options);
    console.log('ApproveWETHRes ', ApproveWETHRes);
    let ApproveCECRes = await contractTokenCEC.approve(contractTokenProxy.address, erc20_balance_cec, options);
    console.log('ApproveCECRes ', ApproveCECRes);

    //check allowance
    let allowance_WBTC = await contractTokenWBTC.allowance(account1, contractTokenProxy.address, options);
    console.log('allowance_WBTC ', allowance_WBTC);
    let allowance_USDT = await contractTokenUSDT.allowance(account1, contractTokenProxy.address, options);
    console.log('allowance_USDT ', allowance_USDT);
    let allowance_WETH = await contractTokenWETH.allowance(account1, contractTokenProxy.address, options);
    console.log('allowance_WETH ', allowance_WETH);
    let allowance_CEC = await contractTokenCEC.allowance(account1, contractTokenProxy.address, options);
    console.log('allowance_CEC ', allowance_CEC);

    let check_pair_result1 = await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenUSDT.address, options);
    console.log('check_pair_wbtc_usdt result ', check_pair_result1);
    let check_pair_result2 = await contractChemixStorage.checkPairExist(contractTokenWETH.address, contractTokenUSDT.address, options);
    console.log('check_pair_weth_usdt result ', check_pair_result2);
    let check_pair_result3 = await contractChemixStorage.checkPairExist(contractTokenCEC.address, contractTokenUSDT.address, options);
    console.log('check_pair_cec_usdt result ', check_pair_result3);
    let check_pair_result4 = await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenCEC.address, options);
    console.log('check_pair_wbtc_cec result ', check_pair_result4);
    let check_pair_result5 = await contractChemixStorage.checkPairExist(contractTokenWETH.address, contractTokenCEC.address, options);
    console.log('check_pair_weth_cec result ', check_pair_result5);

}

main();