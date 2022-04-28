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

    const ethParams = await txParams();
    const gasConf = { gasPrice: ethParams.txGasPrice, gasLimit: ethParams.txGasLimit, value: 0};

    /***
     *  *
     *  * deployTokenCEC:   0xfd4322c6026A761A1ecbD7B5F656FF3C4aCD6fBf
     *  * deployTokenUSDT:   0xe54183F5cB818d2AAaddC25dD03a5687cF527c84
     *  * deployTokenWBTC:   0x0F381a51b032aFbc020856B5E0C764DD910488D2
     *  * deployTokenWETH:   0x35f88BD3A6c2486D5f4115f5eEFF277FCf5278fA
     *  * ^@deployStorage:   0x67A4BCF181314053C6A8410Df3b763Fc15F85041
     *  * deployTokenProxy:   0x700cf11FB9906b38166D586ff2E9Ab390181b265
     *  * deployVault:   0x5254A4A50e5D87a33cF15a477283fa682671509C
     *  * deployChemiMain:   0x98cdEee565d00AC793866B194cB562A6254f4495
     *  *
     *
     * */


    //token
    const contractTokenCEC = await ethers.getContractAt("ChemixPlatform", '0xfd4322c6026A761A1ecbD7B5F656FF3C4aCD6fBf', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0xe54183F5cB818d2AAaddC25dD03a5687cF527c84', chemix_signer)
    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0x0F381a51b032aFbc020856B5E0C764DD910488D2', chemix_signer)
    const contractTokenWETH = await ethers.getContractAt("WrapedEtherum", '0x35f88BD3A6c2486D5f4115f5eEFF277FCf5278fA', chemix_signer)
    //chemix
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0x67A4BCF181314053C6A8410Df3b763Fc15F85041', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0x700cf11FB9906b38166D586ff2E9Ab390181b265', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0x5254A4A50e5D87a33cF15a477283fa682671509C', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0x98cdEee565d00AC793866B194cB562A6254f4495', chemix_signer)

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
    let grantSettleAddr_result2 = await contractVault.grantSettleAddr(account1, gasConf);
    console.log('apply grantSettleAddr_result result ', grantSettleAddr_result2);
    let grantFronzenAddr_result2 = await contractVault.grantFronzenAddr(account1, gasConf);
    console.log('apply grantSettleAddr_result result ', grantSettleAddr_result2);
    let grantCreatePairAddr_result = await contractChemixMain.grantCreatePairAddr(account1, gasConf);
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
    /***
    console.log('start create pair');
    let create_result_WBTC_USDT = await contractChemixMain.createPair(contractTokenWBTC.address, contractTokenUSDT.address, gasConf);
    console.log('create WBTC-USDT pair result ', create_result_WBTC_USDT);
    console.log('start create pair TokenC-TokenCHE');
    let create_result_WETH_CHE = await contractChemixMain.createPair(contractTokenWETH.address, contractTokenUSDT.address, gasConf);
    console.log('create WETH-USDT pair result ', create_result_WETH_CHE);
    await contractChemixMain.createPair(contractTokenCEC.address, contractTokenUSDT.address, gasConf);
    await contractChemixMain.createPair(contractTokenWBTC.address, contractTokenCEC.address, gasConf);
    await contractChemixMain.createPair(contractTokenWETH.address, contractTokenCEC.address, gasConf);
    **/

    //issue token to account1
    let tokenAIssueAcc1Res = await contractTokenWBTC.issue(issueAmountDefault, gasConf);
    await contractTokenWBTC.transfer(account1, issueAmountDefault);

    let tokenBIssueAcc1Res = await contractTokenUSDT.issue(issueAmountDefault, gasConf);
    await contractTokenUSDT.transfer(account1, issueAmountDefault);

    let tokenCIssueAcc1Res = await contractTokenWETH.issue(issueAmountDefault, gasConf);
    await contractTokenWETH.transfer(account1, issueAmountDefault);


    let tokenCHEIssueAcc1Res = await contractTokenCEC.issue(issueAmountDefault, gasConf);
    await contractTokenCEC.transfer(account1, issueAmountDefault);


    let erc20_balance_wbtc = await contractTokenWBTC.balanceOf(account1, gasConf);
    let erc20_balance_weth = await contractTokenWETH.balanceOf(account1, gasConf);
    let erc20_balance_cec = await contractTokenCEC.balanceOf(account1, gasConf);
    let erc20_balance_usdt = await contractTokenUSDT.balanceOf(account1, gasConf);

    console.log('erc20_balance:: wbtc=',erc20_balance_wbtc,'weth=',
        erc20_balance_weth,'cec=',erc20_balance_cec,'usdt=',erc20_balance_usdt);

    //approve permission to chemix
    let ApproveWBTCRes = await contractTokenWBTC.approve(contractTokenProxy.address, erc20_balance_wbtc, gasConf);
    console.log('ApproveWBTCRes ', ApproveWBTCRes);
    let ApproveUSDTRes = await contractTokenUSDT.approve(contractTokenProxy.address, erc20_balance_usdt, gasConf);
    console.log('ApproveUSDTRes ', ApproveUSDTRes);
    let ApproveWETHRes = await contractTokenWETH.approve(contractTokenProxy.address, erc20_balance_weth, gasConf);
    console.log('ApproveWETHRes ', ApproveWETHRes);
    let ApproveCECRes = await contractTokenCEC.approve(contractTokenProxy.address, erc20_balance_cec, gasConf);
    console.log('ApproveCECRes ', ApproveCECRes);

    //check allowance
    let allowance_WBTC = await contractTokenWBTC.allowance(account1, contractTokenProxy.address, gasConf);
    console.log('allowance_WBTC ', allowance_WBTC);
    let allowance_USDT = await contractTokenUSDT.allowance(account1, contractTokenProxy.address, gasConf);
    console.log('allowance_USDT ', allowance_USDT);
    let allowance_WETH = await contractTokenWETH.allowance(account1, contractTokenProxy.address, gasConf);
    console.log('allowance_WETH ', allowance_WETH);
    let allowance_CEC = await contractTokenCEC.allowance(account1, contractTokenProxy.address, gasConf);
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

    //1、createpair
    //2、issuseA to account1
    //3、issuaB to account1
    //4、issuaA to account2
    //5、issuaB to account2
    //6、acount1 approve tokenA to tokenProxy
    //7、acount1 approve tokenB to tokenProxy
    //8、acount2 approve tokenA to tokenProxy
    //9、acount2 approve tokenB to tokenProxy

}

main();