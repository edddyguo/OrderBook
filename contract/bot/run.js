const { ethers, upgrades } = require("hardhat");
const { expect } = require('chai') //断言模块


async function main() {
    //const upgradeContractName = 'ChemixTrade' //升级合约的名称
    //const proxyContractAddress = '0xE41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C' //代理合约的名称
    const proxyContractAddress = '0x4A0C012c4db5801254B47CE142cf916b196FdAdd' //代理合约的名称
    //const proxyContractAddress = '0x5FbDB2315678afecb367f032d93F642f64180aa3' //代理合约的名称

    const DemoUpgrade = await ethers.getContractAt("ChemixTrade",proxyContractAddress)
    let name = await DemoUpgrade.DEXName();
    console.log('name ',name);
    //function newOrder(uint _id,string memory _baseToken, string memory _quoteToken ,uint _amount, uint _price) external returns (string memory){


    let result = await DemoUpgrade.newOrder(1,"BTC","USDT","buy",3,4);
    console.log('result  ',result);
    //0x3b0536683133b13f50f1778971752086ad00d9340e564d790b9c534e0cdd76fc
    let result2 = await DemoUpgrade.listOrders(1);
    console.log('orders  ',result2);
}

main();