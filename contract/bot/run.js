const { ethers, upgrades } = require("hardhat");
const { expect } = require('chai') //断言模块


async function main() {
    //const upgradeContractName = 'ChemixTrade' //升级合约的名称
    const proxyContractAddress = '0xE41d6cA6Ffe32eC8Ceb927c549dFc36dbefe2c0C' //代理合约的名称
    const DemoUpgrade = await ethers.getContractAt("ChemixTrade",proxyContractAddress)
    let name = await DemoUpgrade.DEXName();
    console.log('name ',name);
    //function newOrder(uint _id,string memory _baseToken, string memory _quoteToken ,uint _amount, uint _price) external returns (string memory){


    let result = await DemoUpgrade.newOrder(1,"BTC","USDT",3,4);
    console.log('result  ',result);

    let result2 = await DemoUpgrade.listOrders(1);
    console.log('orders  ',result2);
}

main();