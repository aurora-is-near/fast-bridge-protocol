#[cfg(test)]
mod integration_tests {
    use std::ops::Add;
    use std::time::SystemTime;

    use fast_bridge_common::{TransferDataEthereum, TransferDataNear, TransferMessage};
    use near_sdk::borsh::BorshSerialize;
    use near_sdk::json_types::U128;
    use near_sdk::serde_json::json;
    use near_sdk::{ONE_NEAR, ONE_YOCTO};
    use workspaces::operations::Function;
    use workspaces::{Account, AccountId, Contract};

    use crate::UnlockProof;

    const ETH_BRIDGE_ADDRESS: &str = "6b175474e89094c44da98b954eedeac495271d0f";
    const BRIDGE_WASM_FILEPATH: &str = "../target/wasm32-unknown-unknown/release/fastbridge.wasm";
    const MOCK_PROVER_WASM_FILEPATH: &str =
        "../target/wasm32-unknown-unknown/release/mock_eth_prover.wasm";
    const MOCK_CLIENT_WASM_FILEPATH: &str =
        "../target/wasm32-unknown-unknown/release/mock_eth_client.wasm";
    const MOCK_TOKEN_WASM_FILEPATH: &str =
        "../target/wasm32-unknown-unknown/release/mock_token.wasm";

    #[derive(serde::Serialize)]
    struct InitArgs {
        eth_bridge_contract: String,
        prover_account: Option<String>,
        eth_client_account: Option<String>,
        lock_time_min: String,
        lock_time_max: String,
        eth_block_time: near_sdk::Duration,
        whitelist_mode: bool,
    }

    struct TestData {
        bridge: Contract,
        token: Contract,
        accounts: Vec<Account>,
    }

    async fn deploy_bridge(mut init_input: InitArgs, file_path: &str) -> anyhow::Result<TestData> {
        let worker = workspaces::sandbox().await?;
        let bridge = worker.dev_deploy(&std::fs::read(file_path)?).await?;
        let client = worker
            .dev_deploy(&std::fs::read(MOCK_CLIENT_WASM_FILEPATH)?)
            .await?;
        let prover = worker
            .dev_deploy(&std::fs::read(MOCK_PROVER_WASM_FILEPATH)?)
            .await?;
        let token = worker
            .dev_deploy(&std::fs::read(MOCK_TOKEN_WASM_FILEPATH)?)
            .await?;

        init_input.eth_client_account = Some(
            init_input
                .eth_client_account
                .unwrap_or_else(|| client.id().to_string()),
        );
        init_input.prover_account = Some(
            init_input
                .prover_account
                .unwrap_or_else(|| prover.id().to_string()),
        );
        let result = bridge
            .call("new")
            .args_json(init_input)
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());

        let result = token
            .call("new")
            .args_json(json!({
                "owner_id": bridge.id(),
                "total_supply": "1000",
                "metadata": {
                    "spec": "ft-1.0.0",
                    "name": "Wrapped Near",
                    "symbol": "WNEAR",
                    "decimals": 24
                },
            }))
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());

        let owner = worker.root_account()?;
        let alice = owner
            .create_subaccount("alice")
            .initial_balance(100 * ONE_NEAR)
            .transact()
            .await?
            .into_result()?;

        let result = token
            .call("storage_deposit")
            .args_json(json!({
                "account_id": bridge.id(),
            }))
            .max_gas()
            .deposit(ONE_NEAR)
            .transact()
            .await?;
        assert!(result.is_success());

        let result = token
            .call("mint")
            .args_json(json!({
                "account_id": alice.id(),
                "amount": "1000"
            }))
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());

        let result = client
            .call("set_last_block_number")
            .args_json(json!({
                "block_number": 0,
            }))
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());

        let result = prover
            .call("set_log_entry_verification_status")
            .args_json(json!({
                "verification_status": true,
            }))
            .max_gas()
            .transact()
            .await?;
        assert!(result.is_success());

        Ok(TestData {
            bridge,
            token,
            accounts: [alice].to_vec(),
        })
    }

    async fn get_token_balance(token: &Contract, account_id: &AccountId) -> anyhow::Result<U128> {
        Ok(token
            .view("ft_balance_of")
            .args_json(json!({
                "account_id": account_id,
            }))
            .await?
            .json()?)
    }

    async fn get_bridge_balance(
        bridge: &Contract,
        account_id: &AccountId,
        token_id: &AccountId,
    ) -> anyhow::Result<U128> {
        Ok(bridge
            .view("get_user_balance")
            .args_json(json!({
                "account_id": account_id,
                "token_id": token_id,
            }))
            .await?
            .json()?)
    }

    async fn withdraw_tokens(
        bridge_id: &AccountId,
        account: &Account,
        token_id: &AccountId,
        amount: u128,
        batch_size: u32,
    ) -> Result<workspaces::result::ExecutionFinalResult, workspaces::error::Error> {
        let mut transaction = account.batch(bridge_id);
        for _i in 0..batch_size {
            transaction = transaction.call(
                Function::new("withdraw")
                    .args_json(json!({
                        "token_id": token_id,
                        "amount": amount.to_string(),
                    }))
                    .gas(50 * near_sdk::Gas::ONE_TERA.0),
            );
        }

        transaction.transact().await
    }

    async fn unlock_tokens(
        bridge_id: &AccountId,
        account: &Account,
        nonce: u128,
        batch_size: u32,
    ) -> Result<workspaces::result::ExecutionFinalResult, workspaces::error::Error> {
        let proof = UnlockProof {
            header_data: vec![],
            account_proof: vec![],
            account_data: vec![],
            storage_proof: vec![],
        };
        let proof = near_sdk::base64::encode(proof.try_to_vec().unwrap());
        let mut transaction = account.batch(bridge_id);
        for _i in 0..batch_size {
            transaction = transaction.call(
                Function::new("unlock")
                    .args_json(json!({
                        "nonce": nonce.to_string(),
                        "proof": proof,
                    }))
                    .gas(150 * near_sdk::Gas::ONE_TERA.0),
            );
        }

        transaction.transact().await
    }

    #[tokio::test]
    async fn test_multi_withdraw() -> anyhow::Result<()> {
        let test_data = deploy_bridge(
            InitArgs {
                eth_bridge_contract: ETH_BRIDGE_ADDRESS.to_owned(),
                prover_account: None,
                eth_client_account: None,
                lock_time_min: "1ms".to_owned(),
                lock_time_max: "10h".to_owned(),
                eth_block_time: 0,
                whitelist_mode: false,
            },
            BRIDGE_WASM_FILEPATH,
        )
        .await?;

        // Check init balances
        let alice = &test_data.accounts[0];
        let alice_token_balance = get_token_balance(&test_data.token, alice.id()).await?.0;
        let bridge_balance = get_token_balance(&test_data.token, test_data.bridge.id())
            .await?
            .0;
        assert_eq!(bridge_balance, 1000);
        assert_eq!(alice_token_balance, 1000);
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await
                .unwrap_or(U128(0))
                .0,
            0
        );

        // Transfer tokens from alice to bridge
        let transfer_amount: u128 = 10;
        let result = alice
            .call(test_data.token.id(), "ft_transfer_call")
            .args_json(json!({
                "receiver_id": test_data.bridge.id(),
                "amount": transfer_amount.to_string(),
                "msg": "",
            }))
            .max_gas()
            .deposit(ONE_YOCTO)
            .transact()
            .await?;
        let result: U128 = result.json()?;
        assert_eq!(result.0, transfer_amount);

        // Check account balance after the transfer
        assert_eq!(
            get_token_balance(&test_data.token, alice.id()).await?.0,
            alice_token_balance - transfer_amount
        );
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await?
                .0,
            transfer_amount
        );

        // Call withdraw multiple time with batch transaction
        let withdrawals_batch_size = 3u32;
        let _result = withdraw_tokens(
            test_data.bridge.id(),
            alice,
            test_data.token.id(),
            transfer_amount,
            withdrawals_batch_size,
        )
        .await?;

        // Check account balance after withdraw batch calls
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await?
                .0,
            transfer_amount
        );

        assert_eq!(
            get_token_balance(&test_data.token, alice.id()).await?.0,
            alice_token_balance - transfer_amount
        );

        // Withdraw once
        let withdrawals_batch_size = 1u32;
        let _result = withdraw_tokens(
            test_data.bridge.id(),
            alice,
            test_data.token.id(),
            transfer_amount,
            withdrawals_batch_size,
        )
        .await?;

        // Check acoount balance after withdraw call
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await?
                .0,
            0
        );

        assert_eq!(
            get_token_balance(&test_data.token, alice.id()).await?.0,
            alice_token_balance
        );
        Ok(())
    }

    async fn transfer_tokens(
        bridge: &Account,
        account: &Account,
        token: &Account,
        transfer_amount: u128,
        fee_amount: u128,
    ) -> Result<workspaces::result::ExecutionFinalResult, workspaces::error::Error> {
        let valid_till = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .add(std::time::Duration::from_secs(10))
            .as_nanos()
            .try_into()
            .expect("Can't convert Duration to u64");
        let msg: TransferMessage = TransferMessage {
            valid_till,
            transfer: TransferDataEthereum {
                token_near: token.id().to_string().parse().unwrap(),
                token_eth: [0; 20],
                amount: transfer_amount.into(),
            },
            fee: TransferDataNear {
                token: token.id().to_string().parse().unwrap(),
                amount: fee_amount.into(),
            },
            recipient: [1; 20],
            valid_till_block_height: None,
        };
        let msg = near_sdk::base64::encode(msg.try_to_vec().unwrap());

        let result = account
            .call(token.id(), "ft_transfer_call")
            .args_json(json!({
                "receiver_id": bridge.id(),
                "amount": (transfer_amount + fee_amount).to_string(),
                "msg": msg,
            }))
            .max_gas()
            .deposit(ONE_YOCTO)
            .transact()
            .await?;
        Ok(result)
    }

    #[tokio::test]
    async fn test_multi_unlock() -> anyhow::Result<()> {
        let test_data = deploy_bridge(
            InitArgs {
                eth_bridge_contract: ETH_BRIDGE_ADDRESS.to_owned(),
                prover_account: None,
                eth_client_account: None,
                lock_time_min: "1ms".to_owned(),
                lock_time_max: "10h".to_owned(),
                eth_block_time: 12000000000,
                whitelist_mode: false,
            },
            BRIDGE_WASM_FILEPATH,
        )
        .await?;

        // Check init balances
        let alice = &test_data.accounts[0];
        let alice_token_balance = get_token_balance(&test_data.token, alice.id()).await?.0;
        let bridge_balance = get_token_balance(&test_data.token, test_data.bridge.id())
            .await?
            .0;
        assert_eq!(bridge_balance, 1000);
        assert_eq!(alice_token_balance, 1000);
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await
                .unwrap_or(U128(0))
                .0,
            0
        );

        // Init token transfer twice
        let total_transfer_amount: u128 = 10;
        let total_fee_amount: u128 = 10;
        let result = transfer_tokens(
            test_data.bridge.as_account(),
            alice,
            test_data.token.as_account(),
            total_transfer_amount / 2,
            total_fee_amount / 2,
        )
        .await?;
        let result: U128 = result.json()?;

        assert_eq!(result.0, (total_transfer_amount + total_fee_amount) / 2);

        let result = transfer_tokens(
            test_data.bridge.as_account(),
            alice,
            test_data.token.as_account(),
            total_transfer_amount / 2,
            total_fee_amount / 2,
        )
        .await?;
        let result: U128 = result.json()?;
        assert_eq!(result.0, (total_transfer_amount + total_fee_amount) / 2);

        // Check account balance after the transfer
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await?
                .0,
            0
        );
        assert_eq!(
            get_token_balance(&test_data.token, alice.id()).await?.0,
            alice_token_balance - total_transfer_amount - total_fee_amount
        );

        // Call unlock multiple time with batch transaction
        let unlock_tokens_batch_size = 2u32;
        let _result =
            unlock_tokens(test_data.bridge.id(), alice, 1, unlock_tokens_batch_size).await?;

        // Check account balance after unlock batch calls
        assert_eq!(
            get_bridge_balance(&test_data.bridge, alice.id(), test_data.token.id())
                .await?
                .0,
            (total_transfer_amount + total_fee_amount) / 2,
        );
        assert_eq!(
            get_token_balance(&test_data.token, alice.id()).await?.0,
            alice_token_balance - total_transfer_amount - total_fee_amount
        );

        Ok(())
    }
}