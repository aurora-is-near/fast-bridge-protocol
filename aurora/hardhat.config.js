require("@nomicfoundation/hardhat-chai-matchers");
require("@openzeppelin/hardhat-upgrades");
require("@nomicfoundation/hardhat-verify");

// Replace this private key with your Ropsten account private key
// To export your private key from Metamask, open Metamask and
// go to Account Details > Export Private Key
// Be aware of NEVER putting real Ether into testing accounts
require('dotenv').config();

const AURORA_PRIVATE_KEY = process.env.AURORA_PRIVATE_KEY;
const ETHERSCAN_API_KEY = process.env.ETHERSCAN_API_KEY;

task("deploy", "Deploy aurora fast bridge proxy contract")
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .setAction(async (taskArgs, hre) => {
        const { deploy } = require("./scripts/deploy.js");
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await hre.run("compile");
        await deploy({
            signer,
            nearFastBridgeAccountId: config.nearFastBridgeAccountId,
            auroraEngineAccountId: config.auroraEngineAccountId,
            nativeTokenAccountId: config.nativeTokenAccountId,
            wNearAddress: config.wNearAddress,
            auroraSdkAddress: config.auroraSdkAddress,
            auroraUtilsAddress: config.auroraUtilsAddress
        });
    });

task("upgrade", "Upgrade aurora fast bridge proxy contract")
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .setAction(async (taskArgs, hre) => {
        const { upgrade } = require("./scripts/deploy.js");
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await hre.run("compile");
        await upgrade({
            signer,
            proxyAddress: config.auroraFastBridgeAddress,
            auroraSdkAddress: config.auroraSdkAddress,
            auroraUtilsAddress: config.auroraUtilsAddress
        });
    });

task('register_token', 'Registers a binding of "nearTokenAccountId:auroraTokenAddress" in "AuroraFastBridge" contract.')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nearTokenAccountId', "Token account id on Near")
    .setAction(async taskArgs => {
        const { registerToken, setNativeTokenAccountId } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await registerToken(signer, config, taskArgs.nearTokenAccountId);
    });

task('storage_deposit', 'Puts a storage deposit in "nearTokenAccountId" for the "AuroraFastBridge" implicit NEAR Account ID.')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('fastBridgeAddress', 'Aurora Fast Bridge address')
    .addParam('nearTokenAccountId', "Token account id on Near")
    .setAction(async taskArgs => {
        const { storageDeposit } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await storageDeposit(signer, config, taskArgs.nearTokenAccountId);
    });

task('init_token_transfer', 'Initialize Token Transfer from Aurora to Ethereum')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nearTokenAccountId', "Token account id on Near")
    .addParam('auroraTokenAddress', "Token address on Aurora")
    .addParam('ethTokenAddress', "Token address on Eth")
    .setAction(async taskArgs => {
        const { initTokenTransfer } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await initTokenTransfer(signer, config, taskArgs.nearTokenAccountId, taskArgs.auroraTokenAddress, taskArgs.ethTokenAddress);
    });

task('unlock', 'Unlock tokens on Near')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nonce', 'Nonce of the Fast Bridge transfer')
    .setAction(async taskArgs => {
        const { unlock } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await unlock(signer, config, taskArgs.nonce);
    });

task('get_pending_transfer', 'Get pending transfer by nonce on Near')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nonce', 'Nonce of the Fast Bridge transfer')
    .setAction(async taskArgs => {
        const { get_pending_transfer } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await get_pending_transfer(config, taskArgs.nonce);
    });

task('fast_bridge_withdraw_on_near', 'Withdraw tokens on Near side')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nearTokenAccountId', "Token account id on Near")
    .addParam('tokenAmount', "Withdraw tokens amount")
    .setAction(async taskArgs => {
        const { fast_bridge_withdraw_on_near } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();

        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);
        await fast_bridge_withdraw_on_near(signer, config, taskArgs.nearTokenAccountId, taskArgs.tokenAmount);
    });

task('withdraw_from_implicit_near_account', 'Withdraw tokens to user from Aurora Fast Bridge Implicit Near Account')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nearTokenAccountId', "Token account id on Near")
    .addParam('recipientAddress', "The recipient address to withdraw")
    .setAction(async taskArgs => {
        const { withdraw_from_implicit_near_account } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        if (taskArgs.recipientAddress != "") {
            await withdraw_from_implicit_near_account(signer, config, taskArgs.nearTokenAccountId, taskArgs.recipientAddress);
        } else {
            await withdraw_from_implicit_near_account(signer, config, taskArgs.nearTokenAccountId, signer.address);
        }
    });

task('get_implicit_near_account_id', 'Get near account id for aurora fast bridge contract')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .setAction(async taskArgs => {
        const { get_implicit_near_account_id } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await get_implicit_near_account_id(signer, config);
    });

task('get_token_aurora_address', 'Get aurora token address from aurora fast bridge')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nearTokenAccountId', "Token account id on Near")
    .setAction(async taskArgs => {
        const { get_token_aurora_address } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await get_token_aurora_address(signer, config, taskArgs.nearTokenAccountId);
    });

task('get_balance', 'Get user balance in aurora fast bridge contract')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('nearTokenAccountId', "Token account id on Near")
    .setAction(async taskArgs => {
        const { get_balance } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await get_balance(signer, config, taskArgs.nearTokenAccountId);
    });

task('set_whitelist_mode_for_users', 'Set whitelist mode for users')
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam('userAddress', "User address")
    .setAction(async taskArgs => {
        const { set_whitelist_mode_for_users } = require('./scripts/utils');
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await set_whitelist_mode_for_users(signer, config, taskArgs.userAddress);
    });

task("set_whitelist_mode", "Set whitelist mode")
    .addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
        "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
        "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
    .addParam(
        "enabled",
        "Pass `true` to enable or `false` to disable the whitelist mode"
    )
    .setAction(async (taskArgs) => {
        const { setWhitelistMode } = require("./scripts/utils");
        const [signer] = await hre.ethers.getSigners();
        const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);

        await setWhitelistMode(signer, config);
    });


task("deploy-sdk", "Deploy aurora sdk").setAction(async (_, hre) => {
    const { deploySDK } = require("./scripts/utils");
    const [signer] = await hre.ethers.getSigners();
    
    await hre.run("compile");
    await deploySDK({
        signer,
    });
});

task("set-native-token-account-id", "Set the native token account id")
  .addParam(
    "auroraFastBridgeConfigName",
    "File name without extension for the config " +
      "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
      "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.",
  )
  .setAction(async (taskArgs) => {
    const { setNativeTokenAccountId } = require("./scripts/utils");
    const [signer] = await hre.ethers.getSigners();
    const config = require(
      `./configs/${taskArgs.auroraFastBridgeConfigName}.json`,
    );
    await setNativeTokenAccountId(signer, config);
  });


task('force_increase_balance', 'Force increase users balance')
.addParam("auroraFastBridgeConfigName", "File name without extension for the config " +
    "with dependencies' accounts and addresses used in Aurora Fast Bridge. " +
    "If the CONFIG_NAME is provided, the config with path ./configs/CONFIG_NAME.json will be used.")
.addParam('token', "Token account id on Near")
.addParam('recipient', "Recipient address on Aurora")
.addParam('amount', "Withdraw tokens amount")
.setAction(async taskArgs => {
    const { forceIncreaseBalance } = require('./scripts/utils');
    const [signer] = await hre.ethers.getSigners();

    const config = require(`./configs/${taskArgs.auroraFastBridgeConfigName}.json`);
    await forceIncreaseBalance(signer, config, taskArgs.token, taskArgs.recipient, taskArgs.amount);
});

module.exports = {
    solidity: {
        version: "0.8.17",
        settings: {
            optimizer: {
                enabled: true,
                runs: 75
            },
            metadata: {
                // do not include the metadata hash, since this is machine dependent
                // and we want all generated code to be deterministic
                // https://docs.soliditylang.org/en/v0.8.17/metadata.html
                bytecodeHash: "none"
            }
        }
    },
    networks: {
        mainnet_aurora: {
            url: 'https://mainnet.aurora.dev',
            accounts: [`0x${AURORA_PRIVATE_KEY}`],
            chainId: 1313161554,
            timeout: 100_000_000_000,
        },
        testnet_aurora: {
            url: 'https://testnet.aurora.dev',
            accounts: [`0x${AURORA_PRIVATE_KEY}`],
            chainId: 1313161555,
            timeout: 100_000_000_000
        },
        develop_aurora: {
            url: 'https://develop.rpc.testnet.aurora.dev:8545',
            accounts: [`0x${AURORA_PRIVATE_KEY}`]
        },
        ropsten: {
            url: 'https://rpc.testnet.aurora.dev:8545',
            accounts: [`0x${AURORA_PRIVATE_KEY}`]
        },
        mainnet_enpower: {
            url: "http://powergold.aurora.dev",
            accounts: [`0x${AURORA_PRIVATE_KEY}`],
            chainId: 1313161560,
        },
    },
    etherscan: {
        apiKey: {
          mainnet_aurora: `${ETHERSCAN_API_KEY}`,
          testnet_aurora: `${ETHERSCAN_API_KEY}`,
          mainnet_enpower: `${ETHERSCAN_API_KEY}`
        },
        customChains: [
          {
            network: "mainnet_aurora",
            chainId: 1313161554,
            urls: {
              apiURL: "https://explorer.mainnet.aurora.dev/api",
              browserURL: "https://explorer.mainnet.aurora.dev"
            }
          },
          {
            network: "testnet_aurora",
            chainId: 1313161555,
            urls: {
              apiURL: "https://explorer.testnet.aurora.dev/api",
              browserURL: "https://explorer.testnet.aurora.dev"
            }
          },
          {
            network: "mainnet_enpower",
            chainId: 1313161560,
            urls: {
              apiURL: "https://explorer.powergold.aurora.dev/api",
              browserURL: "https://explorer.powergold.aurora.dev",
            },
          },
        ]
    },
    mocha: {
        timeout: 1000000000000
    }
};
