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
    let account1 = signer[0].address;
    let chemix_signer = signer[0];

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
         * 0227
         * deployTokenA:   0x0C304A4B59107ADd1bb422a27741Db7151559c32
         * deployTokenB:   0x8Db8FFbe335F99778CCbB8DdCa9e210fFd0D54Af
         * deployStorage:   0x53c14905Ba0452eEC44c6AA9Dd294a296aA8dE2c
         * deployTokenProxy:   0x36a4bD8E1dE94A130334397C70aE36641f386Bc3
         * deployVault:   0x1098B7A05c932B0e8b3957f4a8B33cee9Efd724A
         * deployChemiMain:   0x413a559aafA36809F77896Be70284A4C13542f93
         *
         *
         * 0228
         * deployTokenA:   0x12B4e1E58D2EEc9B984A18D7275359E269726Dc2
         * deployTokenB:   0x1B1D8299C787046dE1Be0CCb80aBfeb7Bf126809
         * deployStorage:   0xE5b11BF87f01f952e7Cd268ec710aF0aaE7Ac1aF
         * deployTokenProxy:   0x51751e8d9cB87F8a8A677fF539B2cd4fa45bd435
         * deployVault:   0x5E5849cF979e4984c7785B47A353ddbcA4d82377
         * deployChemiMain:   0xDc776C3FF24A3b4DD11eDB7BCa2474De73856b22
         *
         * 0228:pro
         * deployTokenA:   0x7DBF554b459cFb39C7B92e6AA2FA85Bb1B9aCcF1
         * deployTokenB:   0xAf4984736dAe2e795A8199C01341DA46460a6096
         * deployStorage:   0x241f5bC6CEA90e5c6fd81252804b3A9d714E6c39
         * deployTokenProxy:   0x10CC9D986b8E0a75a1bAbDE209dAEA04872eAA40
         * deployVault:   0x65974E9518cD02Ee99A624366070c85DEe3E36E1
         * deployChemiMain:   0x5304A6d27Cde3427E486b899ab269CA8088e16FC
         *
         * 0301:pro
         * deployTokenA:   0x92177d3e7be191Eb7537299ae1f266de5d2fE939
         * deployTokenB:   0x0eDf2C0379Dba54dDf980cc58666F3698C76f640
         * deployStorage:   0x1464c2aD3402dF09504899870fE87c30b5357FAC
         * deployTokenProxy:   0x4B195824436613d4f3Ab55e2a3f61a6f4B5E29b7
         * deployVault:   0x237577E6e314cD8F4AbC1d49aF997b93d0D37B4c
         * deployChemiMain:   0xc7949C68Fc0012F12f3628a409A29f6Ed35d73aC
         *
         *
         *
         * 0301:pro2
         * deployTokenA:   0x93E139a29b5bfe61Ae34B1D8E526C4Db1A8291ef
         * deployTokenB:   0x0ffB2710A3e25370C987fA52e906459d4c03e105
         * deployStorage:   0xf225989a42Fa37f67235c755526034Da1e0Da0db
         * deployTokenProxy:   0xdB0bb1Aab12d92deDF56a6D55Efcd51289248D10
         * deployVault:   0x45999bf52039320f976b2E541E56c6D8663CFdF2
         * deployChemiMain:   0x24B0e07EBf1cFfa4710a996877307538864E934E
         *
         * deployTokenA:   0x1785f0481CA0a369061802548444b3162B19070b
         * deployTokenB:   0x937Eb6B6d2803e627B06270B732866B9B0E5E71d
         * deployTokenC:   0x75cee65DCf0EA58801779FF716156eEB0bebb2C8
         * deployTokenCHE:   0x0702f6Ce4d63c0F81458F20b566eaC652EA669BF
         * deployStorage:   0xAB07D57aa144c9BCf897E1de54A66629C8F22ba7
         * deployTokenProxy:   0x34d291987a6EaA505015f8b62EDB7b6425BC7183
         * deployVault:   0x9Cb7A3d38641ccC23bFa96Ae12ba6ccA25a886Ee
         * deployChemiMain:   0x9568cd934AcA5C2a21E161928C94Ea1EE4e7A5B5
         *
         * **/



    const contractTokenA = await ethers.getContractAt("BaseToken1",'0x1785f0481CA0a369061802548444b3162B19070b',chemix_signer)
    const contractTokenB = await ethers.getContractAt("QuoteToken1",'0x937Eb6B6d2803e627B06270B732866B9B0E5E71d',chemix_signer)
    const contractTokenC = await ethers.getContractAt("TokenC",'0x75cee65DCf0EA58801779FF716156eEB0bebb2C8',chemix_signer)
    const contractTokenCHE = await ethers.getContractAt("TokenCHE",'0x0702f6Ce4d63c0F81458F20b566eaC652EA669BF',chemix_signer)

    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0xAB07D57aa144c9BCf897E1de54A66629C8F22ba7',chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0x34d291987a6EaA505015f8b62EDB7b6425BC7183',chemix_signer)
    const contractVault = await ethers.getContractAt("Vault",'0x9Cb7A3d38641ccC23bFa96Ae12ba6ccA25a886Ee',chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0x9568cd934AcA5C2a21E161928C94Ea1EE4e7A5B5',chemix_signer)



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
    let create_result_A_B = await contractChemixMain.createPair(contractTokenA.address,contractTokenB.address,options);
    console.log('create pair result ',create_result_A_B);

    console.log('start create pair TokenC-TokenCHE');
    let create_result_CCC_CHE = await contractChemixMain.createPair(contractTokenC.address,contractTokenCHE.address,options);
    console.log('create pair result ',create_result_CCC_CHE);


    //issue tokenA to account1
    //issue tokenB to account1
    let tokenAIssueAcc1Res = await contractTokenA.issue(issueAmountDefault,options);
    console.log('tokenAIssueAcc1Res ',tokenAIssueAcc1Res);
    let tokenBIssueAcc1Res = await contractTokenB.issue(issueAmountDefault,options);
    console.log('tokenAIssueAcc2Res ',tokenBIssueAcc1Res);
    console.log("deployTokenC:  ", contractTokenC.address);
    let tokenCIssueAcc1Res = await contractTokenC.issue(issueAmountDefault,options);
    console.log('tokenCIssueAcc1Res ',tokenCIssueAcc1Res);

    console.log("deployTokenCHE:  ", contractTokenCHE.address);
    let tokenCHEIssueAcc1Res = await contractTokenCHE.issue(issueAmountDefault,options);
    console.log('tokenCHEIssueAcc1Res ',tokenCHEIssueAcc1Res);

    let balanceAcc1 = await contractTokenA.balanceOf(account1,options);
    console.log('balanceA ',balanceAcc1);
    let balanceBcc1 = await contractTokenB.balanceOf(account1,options);
    console.log('balanceB ',balanceBcc1);


    //approve
    let acc1ApproveTokenARes = await contractTokenA.approve(contractTokenProxy.address,balanceAcc1,options);
    console.log('acc1ApproveTokenARes ',acc1ApproveTokenARes);
    let acc1ApproveTokenBRes = await contractTokenB.approve(contractTokenProxy.address,balanceBcc1,options);
    console.log('acc1ApproveTokenBRes ',acc1ApproveTokenBRes);
    let acc1ApproveTokenCRes = await contractTokenC.approve(contractTokenProxy.address,balanceAcc1,options);
    console.log('acc1ApproveTokenCRes ',acc1ApproveTokenCRes);
    let acc1ApproveTokenCHERes = await contractTokenCHE.approve(contractTokenProxy.address,balanceBcc1,options);
    console.log('acc1ApproveTokenCHERes ',acc1ApproveTokenCHERes);

    let allowanceA = await contractTokenA.allowance(account1,contractTokenProxy.address,options);
    console.log('allowanceA ',allowanceA);
    let allowanceB = await contractTokenB.allowance(account1,contractTokenProxy.address,options);
    console.log('allowanceB ',allowanceB);

    let allowanceC = await contractTokenC.allowance(account1,contractTokenProxy.address,options);
    console.log('allowanceC ',allowanceC);
    let allowanceCHE = await contractTokenCHE.allowance(account1,contractTokenProxy.address,options);
    console.log('allowanceCHE ',allowanceCHE);
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