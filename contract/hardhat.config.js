//hardhat项目依赖组件
require("@nomiclabs/hardhat-waffle");
require('@openzeppelin/hardhat-upgrades');

//hardhat项目配置项
module.exports = {
  solidity: "0.8.4",
  networks: {
    tbsc: {
      url: 'https://data-seed-prebsc-1-s1.binance.org:8545', //bsc test net
      accounts: [
        // 0xfAA56B120b8de4597cF20EfF21045a9883e82aad (第14个账户地址及秘钥)
        '0x1b03a06c4a89d570a8f1d39e9ff0be8891f7657898675f11585aa7ec94fe2d12',
        // 0xEA910a9452BDFb5a540F1722B48561a4C2Dc6a6e (第15个账户地址及秘钥)
        '0xa61f2324c70f7c60ec001eee8b8d4255eb5dd673bb2314047ec1314f2adfb84b',
      ]
    },
    mbsc: {
      url: 'https://bsc-dataseed4.ninicoin.io', //bsc main net
      accounts: [
        // 0xfAA56B120b8de4597cF20EfF21045a9883e82aad (第14个账户地址及秘钥)
        '0x1b03a06c4a89d570a8f1d39e9ff0be8891f7657898675f11585aa7ec94fe2d12',
        // 0xEA910a9452BDFb5a540F1722B48561a4C2Dc6a6e (第15个账户地址及秘钥)
        '0xa61f2324c70f7c60ec001eee8b8d4255eb5dd673bb2314047ec1314f2adfb84b',
      ]
    },
    local: {
      url: 'http://127.0.0.1:8545', //本地RPC地址
      //本地区块链账户地址(需要启动运行npx hardhat node命令开启本地开发环境的区块链)
      //这些账户地址和秘钥每次重启区块链都是相同的,并且数据会重置
      accounts: [
        // 0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266 (第一个账户地址及秘钥)
        '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80',
        // 0x70997970c51812dc3a010c7d01b50e0d17dc79c8 (第二个账户地址及秘钥)
        '0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d',
        // 0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc (三个账户地址及秘钥)
        '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a',
        // 0x90f79bf6eb2c4f870365e785982e1f101e93b906 (第四个个账户地址及秘钥)
        '0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6',
        // 0x15d34aaf54267db7d7c367839aaf71a00a2c6a65 (第五个账户地址及秘钥)
        '0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a',
      ]
    }

  }
};
