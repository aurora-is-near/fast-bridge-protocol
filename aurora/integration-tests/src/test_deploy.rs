#[cfg(test)]
pub mod test_deploy {
    use aurora_sdk_integration_tests::{tokio, utils::process, workspaces};
    use std::path::Path;

    pub const TOKEN_SUPPLY: u64 = 1_000_000_000;

    pub async fn compile_near_contracts() {
        let contract_path = Path::new("../../near/");
        let output = tokio::process::Command::new("bash")
            .current_dir(contract_path)
            .args([
                "build_for_tests.sh"
            ])
            .output()
            .await
            .unwrap();
        process::require_success(&output).unwrap();
    }

    pub async fn deploy_mock_token(
        worker: &workspaces::Worker<workspaces::network::Sandbox>,
        user_account_id: &str,
    ) -> workspaces::Contract {
        let contract_path = Path::new("../../near/contracts/");
        let artifact_path =
            contract_path.join("target/wasm32-unknown-unknown/release/mock_token.wasm");
        let wasm_bytes = tokio::fs::read(artifact_path).await.unwrap();
        let mock_token = worker.dev_deploy(&wasm_bytes).await.unwrap();

        mock_token
            .call("new_default_meta")
            .args_json(serde_json::json!({"owner_id": user_account_id, "name": "MockToken", "symbol": "MCT", "total_supply": format!("{}", TOKEN_SUPPLY)}))
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        mock_token
    }

    pub async fn deploy_mock_eth_client(
        worker: &workspaces::Worker<workspaces::network::Sandbox>,
    ) -> workspaces::Contract {
        let contract_path = Path::new("../../near/contracts/");
        let artifact_path =
            contract_path.join("target/wasm32-unknown-unknown/release/mock_eth_client.wasm");
        let wasm_bytes = tokio::fs::read(artifact_path).await.unwrap();
        let mock_eth_client = worker.dev_deploy(&wasm_bytes).await.unwrap();

        mock_eth_client
    }

    pub async fn deploy_mock_eth_prover(
        worker: &workspaces::Worker<workspaces::network::Sandbox>,
    ) -> workspaces::Contract {
        let contract_path = Path::new("../../near/contracts/");
        let artifact_path =
            contract_path.join("target/wasm32-unknown-unknown/release/mock_eth_prover.wasm");
        let wasm_bytes = tokio::fs::read(artifact_path).await.unwrap();
        let mock_eth_prover = worker.dev_deploy(&wasm_bytes).await.unwrap();

        mock_eth_prover
            .call("set_log_entry_verification_status")
            .args_json(serde_json::json!({
                "verification_status": true
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        mock_eth_prover
    }

    pub async fn deploy_near_fast_bridge(
        worker: &workspaces::Worker<workspaces::network::Sandbox>,
        mock_token_account_id: &str,
        mock_eth_client: &str,
        mock_eth_prover: &str,
    ) -> workspaces::Contract {
        let contract_path = Path::new("../../near/contracts/");
        let artifact_path =
            contract_path.join("target/wasm32-unknown-unknown/release/fastbridge.wasm");
        let wasm_bytes = tokio::fs::read(artifact_path).await.unwrap();
        let fast_bridge = worker.dev_deploy(&wasm_bytes).await.unwrap();

        fast_bridge
            .call("new")
            .args_json(serde_json::json!({
                "eth_bridge_contract": "DBE11ADC5F9c821341A837f4810123f495fBFd44",
                "prover_account": mock_eth_prover,
                "eth_client_account": mock_eth_client,
                "lock_time_min": "1s",
                "lock_time_max": "24h",
                "eth_block_time": 12000000000u128,
                "whitelist_mode": true,
                "start_nonce": "0",
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        fast_bridge
            .call("acl_grant_role")
            .args_json(serde_json::json!({
                "account_id": fast_bridge.id().to_string(),
                "role": "WhitelistManager"
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        fast_bridge
            .call("set_token_whitelist_mode")
            .args_json(serde_json::json!({
                "token": mock_token_account_id,
                "mode": "CheckToken"
            }))
            .max_gas()
            .transact()
            .await
            .unwrap()
            .into_result()
            .unwrap();

        fast_bridge
    }
}