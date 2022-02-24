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
     * deployTokenA:   0x3e1A99f4Ebdec4F6Da224D54a4a25b7B1445e1ea
     * deployTokenB:   0x707c73B9425276c0c0adcdd0d1178bB541792049
     * deployStorage:   0xdcac0cd7fC67873f9AfCbaC9e7C8F7A46F5443B8
     * deployTokenProxy:   0xdf7eBFcAdE666c6C7167Ad39229918AD34585e1b
     * deployVault:   0xa122d710C1a9c6b9C2908D25fbeD357144A45552
     * deployChemiMain:   0xC8be8a025D17D21Da7c8533A34696251D4594257
     *
     * */


    const contractTokenA = await ethers.getContractAt("BaseToken1",'0x3e1A99f4Ebdec4F6Da224D54a4a25b7B1445e1ea')
    const contractTokenB = await ethers.getContractAt("QuoteToken1",'0x707c73B9425276c0c0adcdd0d1178bB541792049')
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0xdcac0cd7fC67873f9AfCbaC9e7C8F7A46F5443B8')
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0xdf7eBFcAdE666c6C7167Ad39229918AD34585e1b')
    const contractVault = await ethers.getContractAt("Vault",'0xa122d710C1a9c6b9C2908D25fbeD357144A45552')
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0xC8be8a025D17D21Da7c8533A34696251D4594257')

    //check pai



    let A_alanceOf = await contractVault.balanceOf(contractTokenA.address,account1,options);
    console.log('balanceOfA account1 result ',A_alanceOf);
    let B_alanceOf = await contractVault.balanceOf(contractTokenB.address,account1,options);
    console.log('balanceOfB account1 result ',B_alanceOf);

    let balanceAcc_erc20_A = await contractTokenA.balanceOf(account1,options);
    console.log('balanceA account1 ',balanceAcc_erc20_A);
    let balanceAcc_erc20_B = await contractTokenB.balanceOf(account1,options);
    console.log('balanceB account1 ',balanceAcc_erc20_B);


    let A_alanceOf_account3 = await contractVault.balanceOf(contractTokenA.address,account3,options);
    console.log('balanceOfA account3 result ',A_alanceOf_account3);
    let B_alanceOf_account3 = await contractVault.balanceOf(contractTokenB.address,account3,options);
    console.log('balanceOfB account3 result ',B_alanceOf_account3);

    let balanceAcc_erc20_A_account3 = await contractTokenA.balanceOf(account3,options);
    console.log('balanceA account3 ',balanceAcc_erc20_A_account3);
    let balanceAcc_erc20_B_account3 = await contractTokenB.balanceOf(account3,options);
    console.log('balanceB account3 ',balanceAcc_erc20_B_account3);


}

main();
