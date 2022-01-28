const { ethers, upgrades } = require("hardhat");

async function main() {
    const Demo = await ethers.getContractFactory("Demo");
    const demo = await upgrades.deployProxy(Demo, [100000000000]);
    await demo.deployed();
    console.log("testToken deployed to:", demo.address);
    //0xeEbd54F5E8d3bB1Eeb4c67BB26aFB766A98DAB61
}

main();