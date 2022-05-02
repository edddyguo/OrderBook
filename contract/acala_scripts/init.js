const {ethers, upgrades, network} = require("hardhat");
const {expect} = require('chai')
const {defaultHardhatNetworkHdAccountsConfigParams} = require("hardhat/internal/core/config/default-config");
const {getAccountPath} = require("ethers/lib/utils");
const {networks} = require("../hardhat.config"); //断言模块
const { txParams } = require("./utils/transactionHelper");
const hre = require("hardhat");



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
    const ethParams = await txParams();
    const options = { gasPrice: ethParams.txGasPrice, gasLimit: ethParams.txGasLimit, value: 0};

    /***
     *  *
     * deployTokenCEC:   0x4A0C012c4db5801254B47CE142cf916b196FdAdd
     * deployTokenUSDT:   0xa86785aA400B6b27e0bAD7E1CC7dA425b21E6B69
     * deployTokenWBTC:   0x7E005517EcDf953c05c5E07E844155E007C6285E
     * deployTokenWETH:   0xAB1415967609bE6654a8e1FEDa209275DB1f5B9c
     * deployStorage:   0xb3f1410AA0f358771417a53519B634a50Ee3AB1b
     * deployTokenProxy:   0xf86a0a65435Ab39B355b8FA3651346Dbe8EEe14B
     * deployVault:   0xFe61B257B40D189A311Ef9c1F61BcE78df8F5c18
     * deployChemiMain:   0x65479F56d9c60d11e12441A136eeCE11c4d8f4D6
     *
     * */


    //token
    const contractTokenCEC = await ethers.getContractAt("ChemixPlatform", '0x4A0C012c4db5801254B47CE142cf916b196FdAdd', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0xa86785aA400B6b27e0bAD7E1CC7dA425b21E6B69', chemix_signer)
    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0x7E005517EcDf953c05c5E07E844155E007C6285E', chemix_signer)
    const contractTokenWETH = await ethers.getContractAt("WrapedEtherum", '0xAB1415967609bE6654a8e1FEDa209275DB1f5B9c', chemix_signer)
    //chemix
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0xb3f1410AA0f358771417a53519B634a50Ee3AB1b', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0xf86a0a65435Ab39B355b8FA3651346Dbe8EEe14B', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0xFe61B257B40D189A311Ef9c1F61BcE78df8F5c18', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0x65479F56d9c60d11e12441A136eeCE11c4d8f4D6', chemix_signer)


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
    console.log('start create pair TokenC-TokenCHE');
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