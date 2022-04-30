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

    /***
     * deployTokenCEC:   0x2d45B9c1FfaC0260E9252E19E5392E4eaFC3F0bD
     * deployTokenUSDT:   0x4Cd497271012039E490553E9f2Cc6E7247Bb11dB
     * deployTokenWBTC:   0xC239987208873d693AF21ddd5216D4c163B335de
     * deployTokenWETH:   0x909478f18C066F9a90d173195Fba0795c0C4A7e9
     * deployStorage:   0x7A7f5d417348c005226cD235B00EBDFb91b7eEe0
     * deployTokenProxy:   0xc73d26B38F4Fc627ccf3C1DDc05bE65E8b899d63
     * deployVault:   0xB83d40a9e96D6c911CB1755258E7dF5BD4376D16
     * deployChemiMain:   0x83c751616705D7f61F2bA96e0fD0EDFb4BBBA6A5     *
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

    console.log("start check balance");
    let erc20_balance_cec = await contractTokenCEC.balanceOf(receiver, options);
    let erc20_balance_usdt = await contractTokenUSDT.balanceOf(receiver, options);
    let erc20_balance_wbtc = await contractTokenWBTC.balanceOf(account1, options);
    let erc20_balance_weth = await contractTokenWETH.balanceOf(receiver, options);
    console.log("All balance:erc20_balance_cec=",erc20_balance_cec,
        ",erc20_balance_usdt=",erc20_balance_usdt,
        ",erc20_balance_wbtc=",erc20_balance_wbtc,
        ",erc20_balance_weth=",erc20_balance_weth);


    console.log("start approve3");
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
    let allowance_WBTC = await contractTokenWBTC.allowance(receiver, contractTokenProxy.address, options);
    console.log('allowance_WBTC ', allowance_WBTC);
    let allowance_USDT = await contractTokenUSDT.allowance(receiver, contractTokenProxy.address, options);
    console.log('allowance_USDT ', allowance_USDT);
    let allowance_WETH = await contractTokenWETH.allowance(receiver, contractTokenProxy.address, options);
    console.log('allowance_WETH ', allowance_WETH);
    let allowance_CEC = await contractTokenCEC.allowance(receiver, contractTokenProxy.address, options);
    console.log('allowance_CEC ', allowance_CEC);

}

main();