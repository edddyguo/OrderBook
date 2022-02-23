const { ethers, upgrades } = require("hardhat");
const { expect } = require('chai') //断言模块

/***
 *
 * deployTokenA:   0x5FbDB2315678afecb367f032d93F642f64180aa3
 * deployTokenB:   0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512
 * deployStorage:   0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9
 * deployTokenProxy:   0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0
 * deployVault:   0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9
 * deployChemiMain:   0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
 * */

async function main() {
    //peth
    let account1 = "0x613548d151E096131ece320542d19893C4B8c901"
    //local
    //let account1 = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"

    let account2 = "0x37BA121cdE7a0e24e483364185E80ceF655346DD"
    let account3 = "0xca9B361934fc7A7b07814D34423d665268111726"

    let account_tj = "0x3bB395b668Ff9Cb84e55aadFC8e646Dd9184Da9d"


    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    var options = { gasPrice: 10000000000, gasLimit: 850000, value: 0 };

    /***
     * 21:47
     *
     * deployTokenA:   0xc739cD8920C65d372a0561507930aB6993c33E30
     * deployTokenB:   0x1982C0fC743078a7484bd82AC7A17BDab344308e
     * deployStorage:   0xAfC8a33002B274F43FC56D28D515406966354388
     * deployTokenProxy:   0x913e9d1a60bEb3312472A53CAe1fe64bC4df60e2
     * deployVault:   0x003fDe97E3a0932B2Bc709e952C6C9D73E0E9aE4
     * deployChemiMain:   0x0f48DDFe03827cd5Efb23122B44955c222eCd720
     *
     * */


    const contractTokenA = await ethers.getContractAt("BaseToken1",'0xc739cD8920C65d372a0561507930aB6993c33E30')
    const contractTokenB = await ethers.getContractAt("QuoteToken1",'0x1982C0fC743078a7484bd82AC7A17BDab344308e')
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0xAfC8a33002B274F43FC56D28D515406966354388')
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0x913e9d1a60bEb3312472A53CAe1fe64bC4df60e2')
    const contractVault = await ethers.getContractAt("Vault",'0x003fDe97E3a0932B2Bc709e952C6C9D73E0E9aE4')
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0x0f48DDFe03827cd5Efb23122B44955c222eCd720')

    //check pair

    let authorizeCreatePair = await contractChemixMain.authorizeCreatePair(account1,options);
    console.log('check_pair1 result ',authorizeCreatePair);

    let check_pair_result = await contractChemixStorage.checkPairExist(contractTokenA.address,contractTokenB.address,options);
    console.log('check_pair2 result ',check_pair_result);


    let A_alanceOf = await contractVault.balanceOf(contractTokenA.address,account1,options);
    console.log('balanceOfA result ',A_alanceOf);
    let B_alanceOf = await contractVault.balanceOf(contractTokenB.address,account1,options);
    console.log('balanceOfB result ',B_alanceOf);

    let balanceAcc_erc20_A = await contractTokenA.balanceOf(account1,options);
    console.log('balanceA ',balanceAcc_erc20_A);
    let balanceAcc_erc20_B = await contractTokenB.balanceOf(account1,options);
    console.log('balanceB ',balanceAcc_erc20_B);

    //grantCreatePairAddr
    /***
    let grantCreatePairAddr_result = await contractChemixMain.grantCreatePairAddr(account1,options);
    console.log('grantCreatePairAddr result ',grantCreatePairAddr_result);

    //grantSettleAddr
    let grantSettleAddr_result = await contractVault.grantSettleAddr(account2,options);
    console.log('grantSettleAddr_result result ',grantSettleAddr_result);
     ***/

    /***
    let authorizeSettle_res = await contractVault.authorizeSettle(account2,options);
    console.log('authorizeSettle_res result ',authorizeSettle_res);





    //approve
    let acc1ApproveTokenARes2 = await contractTokenA.approve(contractTokenProxy.address,balanceAcc_erc20_A,options);
    //console.log('acc1ApproveTokenARes ',acc1ApproveTokenARes2);
    let acc1ApproveTokenBRes2 = await contractTokenB.approve(contractTokenProxy.address,balanceAcc_erc20_B,options);
   // console.log('acc1ApproveTokenBRes ',acc1ApproveTokenBRes2);

    let allowanceA2 = await contractTokenA.allowance(account_tj,contractTokenProxy.address,options);
    console.log('allowanceA ',allowanceA2);
    let allowanceB2 = await contractTokenB.allowance(account_tj,contractTokenProxy.address,options);
    console.log('allowanceB ',allowanceB2);
     ***/
    let grantCreatePairAddr_result = await contractChemixMain.grantCreatePairAddr(account1,options);
    console.log('grantCreatePairAddr result ',grantCreatePairAddr_result);

    //grantSettleAddr
    let grantSettleAddr_result = await contractVault.grantSettleAddr(account1,options);
    console.log('grantSettleAddr_result result ',grantSettleAddr_result);


    console.log('start create pair TokenA-TokenB');
    let create_result = await contractChemixMain.createPair(contractTokenA.address,contractTokenB.address,options);
    console.log('create pair result ',create_result);


    //issue tokenA to account1
    //issue tokenB to account1
    let tokenAIssueAcc1Res = await contractTokenA.issue(issueAmountDefault,options);
    console.log('tokenAIssueAcc1Res ',tokenAIssueAcc1Res);
    let tokenBIssueAcc1Res = await contractTokenB.issue(issueAmountDefault,options);
    console.log('tokenAIssueAcc2Res ',tokenBIssueAcc1Res);

    let balanceAcc1 = await contractTokenA.balanceOf(account1,options);
    console.log('balanceA ',balanceAcc1);
    let balanceBcc1 = await contractTokenB.balanceOf(account1,options);
    console.log('balanceB ',balanceBcc1);


    //approve
    let acc1ApproveTokenARes = await contractTokenA.approve(contractTokenProxy.address,balanceAcc1,options);
    console.log('acc1ApproveTokenARes ',acc1ApproveTokenARes);
    let acc1ApproveTokenBRes = await contractTokenB.approve(contractTokenProxy.address,balanceBcc1,options);
    console.log('acc1ApproveTokenBRes ',acc1ApproveTokenBRes);

    let allowanceA = await contractTokenA.allowance(account1,contractTokenProxy.address,options);
    console.log('allowanceA ',allowanceA);
    let allowanceB = await contractTokenB.allowance(account1,contractTokenProxy.address,options);
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