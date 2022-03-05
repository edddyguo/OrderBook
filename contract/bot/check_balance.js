const { ethers, upgrades } = require("hardhat");
const { expect } = require('chai') //断言模块

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

    let account_pj = "0x0a23e267605571ac62f199ebb3f0f649dbf20f7d";


    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    var options = { gasPrice: 10000000000, gasLimit: 850000, value: 0 };

    let signer = await ethers.getSigners();
    let account1 = signer[1].address;
    let chemix_signer = signer[1];
    //let account1 = account_pj;

    /***
     * 21:47
     *
     * deployTokenA:   0x3e1A99f4Ebdec4F6Da224D54a4a25b7B1445e1ea
     * deployTokenB:   0x707c73B9425276c0c0adcdd0d1178bB541792049
     * deployStorage:   0xdcac0cd7fC67873f9AfCbaC9e7C8F7A46F5443B8
     * deployTokenProxy:   0xdf7eBFcAdE666c6C7167Ad39229918AD34585e1b
     * deployVault:   0xa122d710C1a9c6b9C2908D25fbeD357144A45552
     * deployChemiMain:   0xC8be8a025D17D21Da7c8533A34696251D4594257
     *
     * */

    /****

     * pro
     * deployTokenA:   0x7DBF554b459cFb39C7B92e6AA2FA85Bb1B9aCcF1
     * deployTokenB:   0xAf4984736dAe2e795A8199C01341DA46460a6096
     * deployStorage:   0x241f5bC6CEA90e5c6fd81252804b3A9d714E6c39
     * deployTokenProxy:   0x10CC9D986b8E0a75a1bAbDE209dAEA04872eAA40
     * deployVault:   0x65974E9518cD02Ee99A624366070c85DEe3E36E1
     * deployChemiMain:   0x5304A6d27Cde3427E486b899ab269CA8088e16FC
     *
     *pro2
     * deployTokenA:   0x93E139a29b5bfe61Ae34B1D8E526C4Db1A8291ef
     * deployTokenB:   0x0ffB2710A3e25370C987fA52e906459d4c03e105
     * deployStorage:   0xf225989a42Fa37f67235c755526034Da1e0Da0db
     * deployTokenProxy:   0xdB0bb1Aab12d92deDF56a6D55Efcd51289248D10
     * deployVault:   0x45999bf52039320f976b2E541E56c6D8663CFdF2
     * deployChemiMain:   0x24B0e07EBf1cFfa4710a996877307538864E934E
     *
     * deployTokenC:   0x04F2b8d54e4885233e690E9910C57E69e8545C19
     * deployTokenCHE:   0x196B8542FaA5055Cd6c38c8CB563a3749B1bF2Ef
     *
     *
     * deployTokenA:   0x1785f0481CA0a369061802548444b3162B19070b
     * deployTokenB:   0x937Eb6B6d2803e627B06270B732866B9B0E5E71d
     * deployTokenC:   0x75cee65DCf0EA58801779FF716156eEB0bebb2C8
     * deployTokenCHE:   0x0702f6Ce4d63c0F81458F20b566eaC652EA669BF
     * deployStorage:   0xAB07D57aa144c9BCf897E1de54A66629C8F22ba7
     * deployTokenProxy:   0x34d291987a6EaA505015f8b62EDB7b6425BC7183
     * deployVault:   0x9Cb7A3d38641ccC23bFa96Ae12ba6ccA25a886Ee
     * deployChemiMain:   0x9568cd934AcA5C2a21E161928C94Ea1EE4e7A5B5
     * */


    const contractTokenWBTC = await ethers.getContractAt("WrapedBitcoin", '0xD5A0e5F666336732D3dad0552e2E6ae23D937913', chemix_signer)
    const contractTokenUSDT = await ethers.getContractAt("TetherToken", '0x88497793A8fA0d1418087282d491872363E56Ac8', chemix_signer)

    const contractChemixStorage = await ethers.getContractAt("ChemixStorage", '0x87852231D018212905a15CDE4155666143C079f7', chemix_signer)
    const contractTokenProxy = await ethers.getContractAt("TokenProxy", '0x0459768c278ecf3b47114dE7dFcA70497397dAdd', chemix_signer)
    const contractVault = await ethers.getContractAt("Vault", '0x5984F8E1dEDadB954ca69d9EBDF9d9a24368539a', chemix_signer)
    const contractChemixMain = await ethers.getContractAt("ChemixMain", '0xD8CBcc11eDaaAB8b93DEe65bdaD14983cA197B42', chemix_signer)


    /****
    const contractTokenA = await ethers.getContractAt("BaseToken1",'0x93E139a29b5bfe61Ae34B1D8E526C4Db1A8291ef')
    const contractTokenB = await ethers.getContractAt("QuoteToken1",'0x0ffB2710A3e25370C987fA52e906459d4c03e105')
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0xf225989a42Fa37f67235c755526034Da1e0Da0db')
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0xdB0bb1Aab12d92deDF56a6D55Efcd51289248D10')
    const contractVault = await ethers.getContractAt("Vault",'0x45999bf52039320f976b2E541E56c6D8663CFdF2')
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0x24B0e07EBf1cFfa4710a996877307538864E934E')
    ***/
    //check pai


    console.log('balanceOfB account1 result ',account1);

    let A_alanceOf = await contractVault.balanceOf(contractTokenWBTC.address,account1,options);
    console.log('balanceOfA account1 result ',A_alanceOf);
    let B_alanceOf = await contractVault.balanceOf(contractTokenUSDT.address,account1,options);
    console.log('balanceOfB account1 result ',B_alanceOf);

    //let balanceAcc_erc20_A = await contractTokenA.balanceOf(account_tj,options);
    //console.log('balanceA account1 ',balanceAcc_erc20_A);
    //let balanceAcc_erc20_B = await contractTokenB.balanceOf(account_tj,options);
    //console.log('balanceB account1 ',balanceAcc_erc20_B);


    /***
    let A_alanceOf_account3 = await contractVault.balanceOf(contractTokenA.address,account3,options);
    console.log('balanceOfA account3 result ',A_alanceOf_account3);
    let B_alanceOf_account3 = await contractVault.balanceOf(contractTokenB.address,account3,options);
    console.log('balanceOfB account3 result ',B_alanceOf_account3);
     **/
    //let balanceAcc_erc20_A_account3 = await contractTokenA.balanceOf(account3,options);
    //console.log('balanceA account3 ',balanceAcc_erc20_A_account3);
    //let balanceAcc_erc20_B_account3 = await contractTokenB.balanceOf(account3,options);
    //console.log('balanceB account3 ',balanceAcc_erc20_B_account3);



}

main();
