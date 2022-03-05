const {ethers, upgrades, network} = require("hardhat");
const {expect} = require('chai')
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
    var options = {gasPrice: 2000000000, gasLimit: 2950000, value: 0};

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
     *
     *
     * pro
     *
     * dev:
     * deployTokenCEC:   0x3ab24b4147C63DD711445D1a08cd951c3e3a2B30
     * deployTokenUSDT:   0x6BF4842015fc751a8e115aA59F9de0F3A8bb683b
     * deployTokenWBTC:   0xBfb07d8B202e4c4b4e54C17Bc7eE929c297560C0
     * deployTokenWETH:   0x16De11b7B2b64DA139203A5792d18C723a2A274f
     * deployStorage:   0x4a6Ea5c8eDC62bCB2d2A55bba2e7A9f52B8CAb56
     * deployTokenProxy:   0xADf477A879c721F1fdf012581526aab748e8dc1b
     * deployVault:   0x52F68D60F82e9b539c1b18A09F43fE557Dc4Ff93
     * deployChemiMain:   0xcAAd62f33E6da11858f6E7228B2160F5F575Dd9D
     *
     *
     * Solidity compilation finished successfully
     * deployTokenCEC:   0x7CDD8A127660BA96217CE200be0bfEACf13254dA
     * deployTokenUSDT:   0x88497793A8fA0d1418087282d491872363E56Ac8
     * deployTokenWBTC:   0xD5A0e5F666336732D3dad0552e2E6ae23D937913
     * deployTokenWETH:   0xF3a293B1b4DAeb1c599A9Ac50A29c97E4C44d43B
     * deployStorage:   0x87852231D018212905a15CDE4155666143C079f7
     * deployTokenProxy:   0x0459768c278ecf3b47114dE7dFcA70497397dAdd
     * deployVault:   0x5984F8E1dEDadB954ca69d9EBDF9d9a24368539a
     * deployChemiMain:   0xD8CBcc11eDaaAB8b93DEe65bdaD14983cA197B42
     * storageAccessRes:   {
     * **/


    const contractTokenCEC = await ethers.getContractAt("ChemixPlatform", '0x7CDD8A127660BA96217CE200be0bfEACf13254dA', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0x88497793A8fA0d1418087282d491872363E56Ac8', chemix_signer)
    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0xD5A0e5F666336732D3dad0552e2E6ae23D937913', chemix_signer)
    const contractTokenWETH = await ethers.getContractAt("WrapedEtherum", '0xF3a293B1b4DAeb1c599A9Ac50A29c97E4C44d43B', chemix_signer)

    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0x87852231D018212905a15CDE4155666143C079f7', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0x0459768c278ecf3b47114dE7dFcA70497397dAdd', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0x5984F8E1dEDadB954ca69d9EBDF9d9a24368539a', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0xD8CBcc11eDaaAB8b93DEe65bdaD14983cA197B42', chemix_signer)


    let authorizeSettle_res = await contractVault.authorizeSettle(account1, options);
    console.log('authorizeSettle_res result ', authorizeSettle_res);

    let authorizeFronzenAddr = await contractVault.authorizeFronzen(account1, options);
    console.log('authorizeFronzenAddr result ', authorizeFronzenAddr);

    let grantSettleAddr_result2 = await contractVault.grantSettleAddr(account1, options);
    console.log('grantSettleAddr_result result ', grantSettleAddr_result2);


    let grantFronzenAddr_result2 = await contractVault.grantFronzenAddr(account1, options);
    console.log('grantSettleAddr_result result ', grantSettleAddr_result2);

    //check pair

    let authorizeCreatePair = await contractChemixMain.authorizeCreatePair(account1, options);
    console.log('check_pair1 result ', authorizeCreatePair);

    let check_pair_result = await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenUSDT.address, options);
    console.log('check_pair2 result ', check_pair_result);


    let A_alanceOf = await contractVault.balanceOf(contractTokenWBTC.address, account1, options);
    console.log('balanceOfA result ', A_alanceOf);
    let B_alanceOf = await contractVault.balanceOf(contractTokenUSDT.address, account1, options);
    console.log('balanceOfB result ', B_alanceOf);

    let balanceAcc_erc20_A = await contractTokenWBTC.balanceOf(account1, options);
    console.log('balanceA ', balanceAcc_erc20_A);
    let balanceAcc_erc20_B = await contractTokenUSDT.balanceOf(account1, options);
    console.log('balanceB ', balanceAcc_erc20_B);

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
     let acc1ApproveTokenARes2 = await contractTokenWBTC.approve(contractTokenProxy.address,balanceAcc_erc20_A,options);
     //console.log('acc1ApproveTokenARes ',acc1ApproveTokenARes2);
     let acc1ApproveTokenBRes2 = await contractTokenUSDT.approve(contractTokenProxy.address,balanceAcc_erc20_B,options);
     // console.log('acc1ApproveTokenBRes ',acc1ApproveTokenBRes2);

     let allowanceA2 = await contractTokenWBTC.allowance(account_tj,contractTokenProxy.address,options);
     console.log('allowanceA ',allowanceA2);
     let allowanceB2 = await contractTokenUSDT.allowance(account_tj,contractTokenProxy.address,options);
     console.log('allowanceB ',allowanceB2);
     ***/
    let grantCreatePairAddr_result = await contractChemixMain.grantCreatePairAddr(account1, options);
    console.log('grantCreatePairAddr result ', grantCreatePairAddr_result);

    //grantSettleAddr
    let grantSettleAddr_result = await contractVault.grantSettleAddr(account1, options);
    console.log('grantSettleAddr_result result ', grantSettleAddr_result);


    let grantFronzenAddr_result = await contractVault.grantFronzenAddr(account1, options);
    console.log('grantSettleAddr_result result ', grantFronzenAddr_result);


    console.log('start create pair TokenA-TokenB');
    let create_result_A_B = await contractChemixMain.createPair(contractTokenWBTC.address, contractTokenUSDT.address, options);
    console.log('create pair result ', create_result_A_B);
    console.log('start create pair TokenC-TokenCHE');
    let create_result_CCC_CHE = await contractChemixMain.createPair(contractTokenWETH.address, contractTokenUSDT.address, options);
    console.log('create pair result ', create_result_CCC_CHE);
    await contractChemixMain.createPair(contractTokenCEC.address, contractTokenUSDT.address, options);
    await contractChemixMain.createPair(contractTokenWBTC.address, contractTokenCEC.address, options);
    await contractChemixMain.createPair(contractTokenWETH.address, contractTokenCEC.address, options);


    //issue tokenA to account1
    //issue tokenB to account1
    let tokenAIssueAcc1Res = await contractTokenWBTC.issue(issueAmountDefault, options);
    console.log('tokenAIssueAcc1Res ', tokenAIssueAcc1Res);
    await contractTokenWBTC.transfer(account_tj, issueAmountDefault);

    let tokenBIssueAcc1Res = await contractTokenUSDT.issue(issueAmountDefault, options);
    console.log('tokenAIssueAcc2Res ', tokenBIssueAcc1Res);
    await contractTokenUSDT.transfer(account_tj, issueAmountDefault);

    console.log("deployTokenC:  ", contractTokenWETH.address);
    let tokenCIssueAcc1Res = await contractTokenWETH.issue(issueAmountDefault, options);
    console.log('tokenCIssueAcc1Res ', tokenCIssueAcc1Res);
    await contractTokenWETH.transfer(account_tj, issueAmountDefault);


    console.log("deployTokenCHE:  ", contractTokenCEC.address);
    let tokenCHEIssueAcc1Res = await contractTokenCEC.issue(issueAmountDefault, options);
    console.log('tokenCHEIssueAcc1Res ', tokenCHEIssueAcc1Res);
    await contractTokenCEC.transfer(account_tj, issueAmountDefault);


    let balanceAcc1 = await contractTokenWBTC.balanceOf(account1, options);
    console.log('balanceA ', balanceAcc1);
    let balanceBcc1 = await contractTokenUSDT.balanceOf(account1, options);
    console.log('balanceB ', balanceBcc1);


    try {
        await contractTokenCEC.approve(contractTokenProxy.address, balanceBcc1+100, options)
    } catch (e) {
        console.log("", e)
    }

    //approve
    let acc1ApproveTokenARes = await contractTokenWBTC.approve(contractTokenProxy.address, balanceAcc1, options);
    console.log('acc1ApproveTokenARes ', acc1ApproveTokenARes);
    let acc1ApproveTokenBRes = await contractTokenUSDT.approve(contractTokenProxy.address, balanceBcc1, options);
    console.log('acc1ApproveTokenBRes ', acc1ApproveTokenBRes);
    let acc1ApproveTokenCRes = await contractTokenWETH.approve(contractTokenProxy.address, balanceAcc1, options);
    console.log('acc1ApproveTokenCRes ', acc1ApproveTokenCRes);
    let acc1ApproveTokenCHERes = await contractTokenCEC.approve(contractTokenProxy.address, balanceBcc1, options);
    console.log('acc1ApproveTokenCHERes ', acc1ApproveTokenCHERes);

    let allowanceA = await contractTokenWBTC.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceA ', allowanceA);
    let allowanceB = await contractTokenUSDT.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceB ', allowanceB);

    let allowanceC = await contractTokenWETH.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceC ', allowanceC);
    let allowanceCHE = await contractTokenCEC.allowance(account1, contractTokenProxy.address, options);
    console.log('allowanceCHE ', allowanceCHE);
    //


    let authorizeCreatePair1 = await contractChemixMain.authorizeCreatePair(account1, options);
    console.log('check_pair1 authorizeCreatePair ', authorizeCreatePair1);

    let check_pair_result1 = await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenUSDT.address, options);
    console.log('check_pair_wbtc_usdt result ', check_pair_result1);
    let check_pair_result2 = await contractChemixStorage.checkPairExist(contractTokenWETH.address, contractTokenUSDT.address, options);
    console.log('check_pair_weth_usdt result ', check_pair_result2);
    let check_pair_result3 = await contractChemixStorage.checkPairExist(contractTokenCEC.address, contractTokenUSDT.address, options);
    console.log('check_pair_cec_usdt result ', check_pair_result3);
    let check_pair_result4 = await contractChemixStorage.checkPairExist(contractTokenWBTC.address, contractTokenCEC.address, options);
    console.log('check_pair_wbtc_cec result ', check_pair_result4);
    let check_pair_result5 = await contractChemixStorage.checkPairExist(contractTokenWETH.address, contractTokenCEC.address, options);
    console.log('check_pair_weth_cec result ', check_pair_result5);


    //const contractTokenWBTC = await ethers.getContractAt("TokenA",'0xF20e4447DF5D02A9717a1c9a25B8d2FBF973bE56')


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