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
    let account1 = signer[0].address;
    let chemix_signer = signer[0];
    let receiver = signer[0].address;
    let receiver_signer = signer[0];

    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
	const ethParams = await txParams();
    const options = { gasPrice: ethParams.txGasPrice, gasLimit: ethParams.txGasLimit, value: 0};

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
	const contractTokenCEC = await ethers.getContractAt("ChemixPlatform", '0x4A0C012c4db5801254B47CE142cf916b196FdAdd', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0xa86785aA400B6b27e0bAD7E1CC7dA425b21E6B69', chemix_signer)
    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0x7E005517EcDf953c05c5E07E844155E007C6285E', chemix_signer)
    const contractTokenWETH = await ethers.getContractAt("WrapedEtherum", '0xAB1415967609bE6654a8e1FEDa209275DB1f5B9c', chemix_signer)
    //chemix
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0xb3f1410AA0f358771417a53519B634a50Ee3AB1b', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0xf86a0a65435Ab39B355b8FA3651346Dbe8EEe14B', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0xFe61B257B40D189A311Ef9c1F61BcE78df8F5c18', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0x65479F56d9c60d11e12441A136eeCE11c4d8f4D6', chemix_signer)


	/***
	 *
	 *  function newLimitBuyOrder(
        address   baseToken,
        address   quoteToken,
        uint256   limitPrice,
        uint256   orderAmount,
        uint256   numPower
	 *
	 * */
console.log('options ', options);
	let neworder_result = await contractChemixMain.newLimitBuyOrder("0x7E005517EcDf953c05c5E07E844155E007C6285E","0xa86785aA400B6b27e0bAD7E1CC7dA425b21E6B69",123400000,100000,10,options);
    console.log('apply neworder result ', neworder_result);
}

main();
