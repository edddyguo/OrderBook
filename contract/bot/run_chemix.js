const { ethers, upgrades } = require("hardhat");
const { expect } = require('chai') //断言模块

/***
 *
 * deployTokenA:   0xF20e4447DF5D02A9717a1c9a25B8d2FBF973bE56
 * deployTokenB:   0xA7A2a6A3D399e5AD69431aFB95dc86aff3BF871d
 * deployStorage:   0xFFc6817E1c8960b278CCb5e47c2e6D3ae9Fed620
 * deployTokenProxy:   0x357Eb6B854fE982fea32d91340BbdbE5bE7DBCFC
 * deployVault:   0x0563fbAdb1F8bDa9B2E2365826599A26D9f0cb89
 * deployChemixMain:   0x4CF5bd7EB82130763F8EdD0B8Ec44DFa21a5993e
 * */

async function main() {
    let account1 = "0x613548d151E096131ece320542d19893C4B8c901"
    let account2 = "0x37BA121cdE7a0e24e483364185E80ceF655346DD"

    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    var options = { gasPrice: 10000000000, gasLimit: 850000, value: 0 };




    const contractTokenA = await ethers.getContractAt("TokenA",'0xF20e4447DF5D02A9717a1c9a25B8d2FBF973bE56')
    const contractTokenB = await ethers.getContractAt("TokenB",'0xA7A2a6A3D399e5AD69431aFB95dc86aff3BF871d')
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0xFFc6817E1c8960b278CCb5e47c2e6D3ae9Fed620')
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0x357Eb6B854fE982fea32d91340BbdbE5bE7DBCFC')
    const contractVault = await ethers.getContractAt("Vault",'0x0563fbAdb1F8bDa9B2E2365826599A26D9f0cb89')
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0x4CF5bd7EB82130763F8EdD0B8Ec44DFa21a5993e')


    console.log('start create pair TokenA-TokenB');
    let create_result = await contractChemixMain.createPair("0xF20e4447DF5D02A9717a1c9a25B8d2FBF973bE56","0xA7A2a6A3D399e5AD69431aFB95dc86aff3BF871d",options);
    console.log('create pair result ',create_result);

    //issue tokenA to account1
    //issue tokenB to account1
    let tokenAIssueAcc1Res = await contractTokenA.issue(issueAmountDefault,options);
    console.log('tokenAIssueAcc1Res ',tokenAIssueAcc1Res);
    let tokenBIssueAcc1Res = await contractTokenB.issue(issueAmountDefault,options);
    console.log('tokenAIssueAcc2Res ',tokenBIssueAcc1Res);

    let balanceAcc1 = await contractTokenA.balanceOf("0x613548d151E096131ece320542d19893C4B8c901",options);
    console.log('balanceA ',balanceAcc1);
    let balanceBcc1 = await contractTokenB.balanceOf("0x613548d151E096131ece320542d19893C4B8c901",options);
    console.log('balanceB ',balanceBcc1);


    //approve
    let acc1ApproveTokenARes = await contractTokenA.approve(contractTokenProxy.address,balanceAcc1,options);
    console.log('acc1ApproveTokenARes ',acc1ApproveTokenARes);
    let acc1ApproveTokenBRes = await contractTokenB.approve(contractTokenProxy.address,balanceBcc1,options);
    console.log('acc1ApproveTokenBRes ',acc1ApproveTokenBRes);

    let allowanceA = await contractTokenA.allowance("0x613548d151E096131ece320542d19893C4B8c901",contractTokenProxy.address,options);
    console.log('allowanceA ',allowanceA);
    let allowanceB = await contractTokenB.allowance("0x613548d151E096131ece320542d19893C4B8c901",contractTokenProxy.address,options);
    console.log('allowanceB ',allowanceB);
    //



    //const contractTokenA = await ethers.getContractAt("TokenA",'0xF20e4447DF5D02A9717a1c9a25B8d2FBF973bE56')



    //function newOrder(uint _id,string memory _baseToken, string memory _quoteToken ,uint _amount, uint _price) external returns (string memory){
    /***
    let result = await DemoUpgrade.newOrder(1,"BTC","USDT","buy",3,4);
    console.log('result  ',result);
    //0x3b0536683133b13f50f1778971752086ad00d9340e564d790b9c534e0cdd76fc
    let result2 = await DemoUpgrade.listOrders(1);
    console.log('orders  ',result2);
     ***/

    //1、createpair
    //2、issuseA to account1
    //3、issuaB to account1
    //4、issuaA to account2
    //5、issuaB to account2
    //6、acount1 approve tokenA to tokenProxy
    //7、acount1 approve tokenB to tokenProxy
    //8、acount2 approve tokenA to tokenProxy
    //9、acount2 approve tokenB to tokenProxy


}

main();