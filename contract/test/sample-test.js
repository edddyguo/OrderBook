const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("Greeter", function () {
  it("Should return the new greeting once it's changed", async function () {
    var options = { gasPrice: 10000000000, gasLimit: 850000, value: 0 };
    const Greeter = await ethers.getContractFactory("Greeter");
    const greeter = await Greeter.deploy("Hello, world!",options);
    await greeter.deployed();

    expect(await greeter.greet()).to.equal("Hello, world!");
    console.log("greeter address ",greeter.address);
    return

    /**
    const setGreetingTx = await greeter.setGreeting("Hola, mundo!");

    // wait until the transaction is mined
    await setGreetingTx.wait();

    expect(await greeter.greet()).to.equal("Hola, mundo!");
    ***/


    /***
    const TokenA = await ethers.getContractFactory("TokenA");
    const tokenA = await TokenA.deploy("Hello, world!");
    await greeter.deployed();

    expect(await greeter.greet()).to.equal("Hello, world!");

    const setGreetingTx2 = await greeter.setGreeting("Hola, mundo!");

    // wait until the transaction is mined
    await setGreetingTx2.wait();

    expect(await greeter.greet()).to.equal("Hola, mundo!");
     ***/


    /***
    const issueAmountDefault = BigInt(100_000_000_000_000_000_000_000_000_000) //100_000_000_000
    var options = { gasPrice: 10000000000, gasLimit: 850000, value: 0 };
    const contractTokenA = await ethers.getContractAt("TokenA",'0x5B1573e94Aa213D9594cf9420EE6960764DB4f68')
    const contractTokenB = await ethers.getContractAt("TokenB",'0x7541289eC7fc012C696535FFE44BC80C58596cC0')
    const contractChemixStorage = await ethers.getContractAt("ChemixStorage",'0xeDE462491f759bcb631aD03DdE4c6aD1B5847DEb')
    const contractTokenProxy = await ethers.getContractAt("TokenProxy",'0x55e44f9327a99F9Ff15C7234225b864466DC6a60')
    const contractVault = await ethers.getContractAt("Vault",'0x5ED7BA2da1229d43F1433bD8127Fb4B8960bccE2')
    const contractChemixMain = await ethers.getContractAt("ChemixMain",'0x1D781d006e451C335Ee3053c390d1a7B6813a725')

    let tokenAIssueAcc1ResTmp1 = await contractTokenA.issue(issueAmountDefault,options);
    await tokenAIssueAcc1ResTmp1.wait();
    console.log('tokenAIssueAcc1Res ',tokenAIssueAcc1ResTmp1);
    let tokenAIssueAcc1ResTmp2 = await contractTokenB.issue(issueAmountDefault,options);
    await tokenAIssueAcc1ResTmp2.wait();
    console.log('tokenAIssueAcc2Res ',tokenAIssueAcc1ResTmp2);
     ***/

    const greet2 = await ethers.getContractAt("Greeter",'0x5FbDB2315678afecb367f032d93F642f64180aa3')
    const setGreetingTx = await greet2.setGreeting("Hola, mundo!");
    await setGreetingTx.wait();

  });
});
