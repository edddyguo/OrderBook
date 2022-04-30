// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// When running the script with `npx hardhat run <script>` you'll find the Hardhat
// Runtime Environment's members available in the global scope.
const hre = require("hardhat");
const {address} = require("hardhat/internal/core/config/config-validation");
const {defaultHardhatNetworkHdAccountsConfigParams} = require("hardhat/internal/core/config/default-config");
const {networks} = require("../hardhat.config");
const {ethers} = require("hardhat"); //断言模块
const { txParams } = require("./utils/transactionHelper");

async function main() {
    const ethParams = await txParams();
    const gasConf = {
        gasPrice: ethParams.txGasPrice,
        gasLimit: ethParams.txGasLimit,
    };

    let signer = await ethers.getSigners();
    let account1 = signer[0].address;
    let chemix_signer = signer[0];

    const tokenCEC = await hre.ethers.getContractFactory("ChemixPlatform",chemix_signer);
    const tokenUSDT = await hre.ethers.getContractFactory("TetherToken",chemix_signer);
    const TokenWBTC = await hre.ethers.getContractFactory("WrapedBitcoin",chemix_signer);
    const TokenWETH = await hre.ethers.getContractFactory("WrapedEtherum",chemix_signer);

    //deploy token
    const deployTokenCEC = await tokenCEC.deploy();
    const deployTokenUSDT = await tokenUSDT.deploy();
    const deployTokenWBTC = await TokenWBTC.deploy();
    const deployTokenWETH = await TokenWETH.deploy();
    await deployTokenCEC.deployed();
    await deployTokenUSDT.deployed();
    await deployTokenWBTC.deployed();
    await deployTokenWETH.deployed();
    console.log("deployTokenCEC:  ", deployTokenCEC.address);
    console.log("deployTokenUSDT:  ", deployTokenUSDT.address);
    console.log("deployTokenWBTC:  ", deployTokenWBTC.address);
    console.log("deployTokenWETH:  ", deployTokenWETH.address);

    //deploy chemix
    const chemixStorage = await hre.ethers.getContractFactory("ChemixStorage",chemix_signer);
    const tokenProxy = await hre.ethers.getContractFactory("TokenProxy",chemix_signer);
    const vault = await hre.ethers.getContractFactory("Vault",chemix_signer);
    const deployTokenProxy = await tokenProxy.deploy();
    const deployStorage = await chemixStorage.deploy();
    const deployVault = await vault.deploy(deployTokenProxy.address,deployStorage.address);
    console.log("deployStorage:  ", deployStorage.address);
    console.log("deployTokenProxy:  ", deployTokenProxy.address);
    console.log("deployVault:  ", deployVault.address);


    //chemix main
    let feeTo = "0xca9B361934fc7A7b07814D34423d665268111726";
    const chemixMain = await hre.ethers.getContractFactory("ChemixMain",chemix_signer);
    const deployChemiMain = await chemixMain.deploy(deployVault.address,deployStorage.address,feeTo,0);
    console.log("deployChemiMain:  ", deployChemiMain.address);

    //grantAccess
    let storageAccessRes= await deployStorage.grantAccess(deployChemiMain.address);
    console.log("storageAccessRes:  ", storageAccessRes);

    let valutAccessRes= await deployVault.grantAccess(deployChemiMain.address);
    console.log("valutAccessRes:  ", valutAccessRes);

    let proxyAccessVaultRes= await deployTokenProxy.grantAccess(deployVault.address);
    console.log("proxyAccessVaultRes:  ", proxyAccessVaultRes);

    let proxyAccessChemixRes= await deployTokenProxy.grantAccess(deployChemiMain.address);
    console.log("proxyAccessChemixRes:  ", proxyAccessChemixRes);

}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(error);
        process.exit(1);
    });
