const {ethers, upgrades, network} = require("hardhat");
const {expect} = require('chai')
const {defaultHardhatNetworkHdAccountsConfigParams} = require("hardhat/internal/core/config/default-config");
const {getAccountPath} = require("ethers/lib/utils");
const {networks} = require("../hardhat.config"); //断言模块
const { txParams } = require("./utils/transactionHelper");




async function main() {
    //peth
    //let account1 = "0x613548d151E096131ece320542d19893C4B8c901"
    let account2 = "0x37BA121cdE7a0e24e483364185E80ceF655346DD"
    let account3 = "0xca9B361934fc7A7b07814D34423d665268111726"
    let account4 = "0xF668b864756a2fB53b679bb13e0F9AB2d9C5fEE0"
    let account_tj = "0x3bB395b668Ff9Cb84e55aadFC8e646Dd9184Da9d"


    let signer = await ethers.getSigners();
    //let account1 = signer[0].address;
    //let chemix_signer = signer[0];
    let account1 = signer[1].address;
    let chemix_signer = signer[1];
    let receiver = signer[1].address;
    let receiver_signer = signer[1];

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
     *    ^@deployTokenCEC:   0x4A0C012c4db5801254B47CE142cf916b196FdAdd
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

    console.log("start check balance");
    let erc20_balance_cec = await contractTokenCEC.balanceOf(receiver, gasConf);
    let erc20_balance_usdt = await contractTokenUSDT.balanceOf(receiver, gasConf);
    let erc20_balance_wbtc = await contractTokenWBTC.balanceOf(account1, options);
    let erc20_balance_weth = await contractTokenWETH.balanceOf(receiver, gasConf);
    console.log("All balance:erc20_balance_cec=",erc20_balance_cec,
        ",erc20_balance_usdt=",erc20_balance_usdt,
        ",erc20_balance_wbtc=",erc20_balance_wbtc,
        ",erc20_balance_weth=",erc20_balance_weth);


    console.log("start approve3");
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
    let allowance_WBTC = await contractTokenWBTC.allowance(receiver, contractTokenProxy.address, gasConf);
    console.log('allowance_WBTC ', allowance_WBTC);
    let allowance_USDT = await contractTokenUSDT.allowance(receiver, contractTokenProxy.address, gasConf);
    console.log('allowance_USDT ', allowance_USDT);
    let allowance_WETH = await contractTokenWETH.allowance(receiver, contractTokenProxy.address, gasConf);
    console.log('allowance_WETH ', allowance_WETH);
    let allowance_CEC = await contractTokenCEC.allowance(receiver, contractTokenProxy.address, gasConf);
    console.log('allowance_CEC ', allowance_CEC);

}

main();