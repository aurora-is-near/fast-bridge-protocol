set -e

BRIDGE_ACCOUNT=fastbridge.testnet
UNLOCK_ACCOUNT=""
TRANSFER_NONCE=1
RAINBOW_BRIDGE_INDEX_JS_PATH="$HOME/aurora/rainbow-bridge/cli/index.js"
ETH_CONTRACT_ADDRESS="0x00763f30eEB0eEF506907e18f2a6ceC2DAb30Df8"
ETH_RPC_URL="https://goerli.infura.io/v3/$RPC_KEY"
export NEAR_ENV=testnet

TRANSFER_MSG=($(near view $BRIDGE_ACCOUNT get_pending_transfer '{"id": "'"$TRANSFER_NONCE"'"}'))
TRANSFER_MSG=${TRANSFER_MSG[@]:6:24}
echo "\n-----------------------------\n" 
STORAGE_KEY_OUTPUT=($(cargo run --manifest-path utils/Cargo.toml -- get-transfer-storage-key -n $TRANSFER_NONCE -m "$TRANSFER_MSG"))
BLOCK_NUMBER=${STORAGE_KEY_OUTPUT[1]}
STORAGE_KEY="0x"${STORAGE_KEY_OUTPUT[3]}
echo "\n-----------------------------\n" 
JSON_PROOF=$($RAINBOW_BRIDGE_INDEX_JS_PATH eth-to-near-find-storage-proof $ETH_CONTRACT_ADDRESS $STORAGE_KEY $BLOCK_NUMBER --eth-node-url $ETH_RPC_URL)
ENCODED_UNLOCK_PROOF=$(cargo run --manifest-path utils/Cargo.toml -- encode-unlock-proof -p $JSON_PROOF | tail -n 1)
echo "\n-----------------------------\n" 

near call $BRIDGE_ACCOUNT unlock "{\"nonce\":\"$TRANSFER_NONCE\", \"proof\": $ENCODED_UNLOCK_PROOF}" --account-id $UNLOCK_ACCOUNT --depositYocto 1 --gas 300000000000000
