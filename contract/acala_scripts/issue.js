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
    let account_tj = "0x0085560b24769dAC4ed057F1B2ae40746AA9aAb6"


    let signer = await ethers.getSigners();
    //let account1 = signer[0].address;
    //let chemix_signer = signer[0];
    let account1 = signer[0].address;
    let chemix_signer = signer[0];
    //let receiver = signer[1].address;
    let receiver = signer[1];
    let receiver_signer = signer[1];

    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    const ethParams = await txParams();
    const options = { gasPrice: ethParams.txGasPrice, gasLimit: ethParams.txGasLimit, value: 0};
    /***
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

    //vault内的balance和erc20的balance
    let A_alanceOf = await contractVault.balanceOf(contractTokenWBTC.address, receiver, options);
    console.log('Balance Of  Vault WBTC', A_alanceOf);
    let B_alanceOf = await contractVault.balanceOf(contractTokenUSDT.address, receiver, options);
    console.log('Balance Of  Vault USDT', B_alanceOf);
    let balanceAcc_erc20_A = await contractTokenWBTC.balanceOf(receiver, options);
    console.log('Erc20 BalanceA ', balanceAcc_erc20_A);
    let balanceAcc_erc20_B = await contractTokenUSDT.balanceOf(receiver, options);
    console.log('Erc20 BalanceB ', balanceAcc_erc20_B);

    //issue token to account1
    let tokenAIssueAcc1Res = await contractTokenWBTC.issue(issueAmountDefault, options);
    await contractTokenWBTC.transfer(receiver, issueAmountDefault);

    let tokenBIssueAcc1Res = await contractTokenUSDT.issue(issueAmountDefault, options);
    await contractTokenUSDT.transfer(receiver, issueAmountDefault);

    let tokenCIssueAcc1Res = await contractTokenWETH.issue(issueAmountDefault, options);
    await contractTokenWETH.transfer(receiver, issueAmountDefault);


    let tokenCHEIssueAcc1Res = await contractTokenCEC.issue(issueAmountDefault, options);
    await contractTokenCEC.transfer(receiver, issueAmountDefault);


    let erc20_balance_wbtc = await contractTokenWBTC.balanceOf(receiver, options);
    let erc20_balance_weth = await contractTokenWETH.balanceOf(receiver, options);
    let erc20_balance_cec = await contractTokenCEC.balanceOf(receiver, options);
    let erc20_balance_usdt = await contractTokenUSDT.balanceOf(receiver, options);

    console.log('erc20_balance:: wbtc=',erc20_balance_wbtc,'weth=',
        erc20_balance_weth,'cec=',erc20_balance_cec,'usdt=',erc20_balance_usdt);
}

main();
