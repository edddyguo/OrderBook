const { ethers, upgrades, network} = require("hardhat");
const { expect } = require('chai')
const {defaultHardhatNetworkHdAccountsConfigParams} = require("hardhat/internal/core/config/default-config");
const {getAccountPath} = require("ethers/lib/utils");
const {networks} = require("../hardhat.config"); //断言模块

/***
 *
 * deployTokenA:   0x3e1A99f4Ebdec4F6Da224D54a4a25b7B1445e1ea
 * deployTokenB:   0x707c73B9425276c0c0adcdd0d1178bB541792049
 * deployStorage:   0xdcac0cd7fC67873f9AfCbaC9e7C8F7A46F5443B8
 * deployTokenProxy:   0xdf7eBFcAdE666c6C7167Ad39229918AD34585e1b
 * deployVault:   0xa122d710C1a9c6b9C2908D25fbeD357144A45552
 * deployChemiMain:   0xC8be8a025D17D21Da7c8533A34696251D4594257
 * */

async function main() {
    //peth
    //let account1 = "0x613548d151E096131ece320542d19893C4B8c901"
    //local
    //let account1 = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"

    let account2 = "0x37BA121cdE7a0e24e483364185E80ceF655346DD"
    let account3 = "0xca9B361934fc7A7b07814D34423d665268111726"

    let account_tj = "0x3bB395b668Ff9Cb84e55aadFC8e646Dd9184Da9d"



    let signer = await ethers.getSigners();
    //let account1 = signer[0].address;
    //let chemix_signer = signer[0];
    let account1 = signer[1].address;
    let chemix_signer = signer[1];

    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    var options = { gasPrice: 10000000000, gasLimit: 850000, value: 0 };

    /***
     * 21:47
     *
     * deployTokenA:   0x34f12975Bb2e7e1e87A0e59642B960C34f10E1A2
     * deployTokenB:   0xFF013CBd3eF16f70ED32f0c080820f30B5dbe978
     * deployStorage:   0xc973A1210Eb6dACd86Fe4dB7744a3A4b8dB6df29
     * deployTokenProxy:   0x297ADc769890dE02DcA27d1ba4e805d6bC46ab3e
     * deployVault:   0x4011080e00A75dd049fDA989769e16e266978e15
     * deployChemiMain:   0xc271117Bd55fcDe1374FaA62bC4F2E48059ddE1a
     *
     * */

    //dev
        /**
         * deployTokenA:   0x78F8D152Dc041E6Aa027342A12D19EF9ecf5038a
         * deployTokenB:   0xCB40288aF19767c0652013D3072e0Dd983d0cFFE
         * deployStorage:   0x7fAb33f49eB76730962762E2B3eAb8C5ca6936EE
         * deployTokenProxy:   0x0b4E525C0A237D2A9Fe8E6173B3D709e605aC141
         * deployVault:   0x36E1C5B7DBa8ab55E4e82bD627d3D81e5D0FaD99
         * deployChemiMain:   0xCcC6515C3d15b4Fd8ef2c6654d18eB81af5aB7F2
         *
         *
         * deployTokenA:   0x38F52517e6642fB1933E7A6A3a34fEa35372eD32
         * deployTokenB:   0x719d36AB3752aa2d0311637B79B480C00A8f83fC
         * deployStorage:   0x2b5a5390170805878c44e3813EbC1f0e48aB2953
         * deployTokenProxy:   0xdFC403273AE6dc9993f58E8e2d15D48D4AAA5Ff5
         * deployVault:   0xA45dd22d573314Fd85b5935B542C236A2dB72534
         * deployChemiMain:   0xEba68dCF72f4601220c4CB576132f7FE3AE25853
         *
         * **/



    const contractTokenA = await ethers.getContractAt("BaseToken1",'0x38F52517e6642fB1933E7A6A3a34fEa35372eD32',chemix_signer)
    const contractTokenB = await ethers.getContractAt("QuoteToken1",'0x719d36AB3752aa2d0311637B79B480C00A8f83fC',chemix_signer)
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0x2b5a5390170805878c44e3813EbC1f0e48aB2953',chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0xdFC403273AE6dc9993f58E8e2d15D48D4AAA5Ff5',chemix_signer)
    const contractVault = await ethers.getContractAt("Vault",'0xA45dd22d573314Fd85b5935B542C236A2dB72534',chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0xEba68dCF72f4601220c4CB576132f7FE3AE25853',chemix_signer)



    let authorizeSettle_res = await contractVault.authorizeSettle(account1,options);
    console.log('authorizeSettle_res result ',authorizeSettle_res);

    let authorizeFronzenAddr = await contractVault.authorizeFronzen(account1,options);
    console.log('authorizeFronzenAddr result ',authorizeFronzenAddr);

    let grantSettleAddr_result2 = await contractVault.grantSettleAddr(account1,options);
    console.log('grantSettleAddr_result result ',grantSettleAddr_result2);


    let grantFronzenAddr_result2 = await contractVault.grantFronzenAddr(account1,options);
    console.log('grantSettleAddr_result result ',grantSettleAddr_result2);

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


    let grantFronzenAddr_result = await contractVault.grantFronzenAddr(account1,options);
    console.log('grantSettleAddr_result result ',grantFronzenAddr_result);


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