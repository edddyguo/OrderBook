const { ethers, upgrades } = require("hardhat");
const { expect } = require('chai') //断言模块


async function main() {
    const upgradeContractName = 'Demo_v2' //升级合约的名称
    const proxyContractAddress = '0x0165878A594ca255338adfa4d48449f69242Eb8F' //代理合约的名称
    const DemoUpgrade = await ethers.getContractFactory(upgradeContractName)
    console.log('Upgrading Demo...')
    let demo_v2 = await upgrades.upgradeProxy(proxyContractAddress, DemoUpgrade);
    console.log('Upgrading success demo_v2 ',demo_v2.address)
    await demo_v2.increment10()
    const newx = (await demo_v2.x()).toString() //获取存储的新值
    console.log('newx ',newx)


    /***
     *
     * const upgradeContractName = 'DemoV2' //升级合约的名称
     const proxyContractAddress = this.demo.address //代理合约的名称
     const DemoUpgrade = await ethers.getContractFactory(upgradeContractName) //工厂合约
     const demoV2 = await upgrades.upgradeProxy(proxyContractAddress, DemoUpgrade) //升级合约
     await this.demo.setScore(800) //设置score为800
     await demoV2.increment() //计数器+1
     const newScore = (await this.demo.score()).toString() //获取存储的新值
     expect(newScore).to.be.equal('801') //断言结果为801
     * */
    //await this.demo.setScore(800)
}

main();