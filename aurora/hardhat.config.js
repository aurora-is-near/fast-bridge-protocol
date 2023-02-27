require("@nomiclabs/hardhat-waffle");
// Replace this private key with your Ropsten account private key
// To export your private key from Metamask, open Metamask and
// go to Account Details > Export Private Key
// Be aware of NEVER putting real Ether into testing accounts
require('dotenv').config();

const AURORA_PRIVATE_KEY = process.env.AURORA_PRIVATE_KEY;

task('init_near_contract', 'Init the Aurora Fast Bridge Contract on NEAR')
    .addParam('fastBridgeAddress', 'Eth address of Aurora Fast Bridge')
    .setAction(async taskArgs => {
        const { initNearContract } = require('./scripts/utils');
        await initNearContract(hre.ethers.provider, taskArgs.fastBridgeAddress);
    });

module.exports = {
    solidity: "0.8.17",
    networks: {
        testnet_aurora: {
            url: 'https://testnet.aurora.dev',
            accounts: [`0x${AURORA_PRIVATE_KEY}`]
        },
        develop_aurora: {
            url: 'https://develop.rpc.testnet.aurora.dev:8545',
            accounts: [`0x${AURORA_PRIVATE_KEY}`]
        },
        ropsten: {
            url: 'https://rpc.testnet.aurora.dev:8545',
            accounts: [`0x${AURORA_PRIVATE_KEY}`]
        }
    }
};
