const { ethers, upgrades } = require("hardhat");

async function main() {
    /****
    const Demo = await ethers.getContractFactory("Demo");
    const demo = await upgrades.deployProxy(Demo, [100000000000]);
    await demo.deployed();
    console.log("testToken deployed to:", demo.address);
     ***/
    const upgradeContractName = 'Demo_v2' //升级合约的名称
    const proxyContractAddress = '0x0165878A594ca255338adfa4d48449f69242Eb8F' //代理合约的名称
    const DemoUpgrade = await ethers.getContractFactory(upgradeContractName)
    console.log('Upgrading Demo...')
    let demo_v2 = await upgrades.upgradeProxy(proxyContractAddress, DemoUpgrade)
    console.log('Upgrading success demo_v2 ',demo_v2.address)

}

main();