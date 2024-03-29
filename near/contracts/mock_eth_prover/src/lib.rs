use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{near_bindgen, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct EthProver {
    log_entry_verification_status: bool,
}

#[near_bindgen]
impl EthProver {
    #[allow(clippy::too_many_arguments, unused_variables)]
    #[result_serializer(borsh)]
    pub fn verify_log_entry(
        &self,
        #[serializer(borsh)] log_index: u64,
        #[serializer(borsh)] log_entry_data: Vec<u8>,
        #[serializer(borsh)] receipt_index: u64,
        #[serializer(borsh)] receipt_data: Vec<u8>,
        #[serializer(borsh)] header_data: Vec<u8>,
        #[serializer(borsh)] proof: Vec<Vec<u8>>,
        #[serializer(borsh)] skip_bridge_call: bool,
    ) -> PromiseOrValue<bool> {
        PromiseOrValue::Value(self.log_entry_verification_status)
    }

    #[allow(clippy::too_many_arguments, unused_variables)]
    #[result_serializer(borsh)]
    pub fn verify_storage_proof(
        &self,
        #[serializer(borsh)] header_data: Vec<u8>,
        #[serializer(borsh)] account_proof: Vec<Vec<u8>>,
        #[serializer(borsh)] contract_address: Vec<u8>,
        #[serializer(borsh)] account_state: Vec<u8>,
        #[serializer(borsh)] storage_key_hash: Vec<u8>,
        #[serializer(borsh)] storage_proof: Vec<Vec<u8>>,
        #[serializer(borsh)] value: Vec<u8>,
        #[serializer(borsh)] min_header_height: Option<u64>,
        #[serializer(borsh)] max_header_height: Option<u64>,
        #[serializer(borsh)] skip_bridge_call: bool,
    ) -> PromiseOrValue<bool> {
        PromiseOrValue::Value(self.log_entry_verification_status)
    }

    pub fn set_log_entry_verification_status(&mut self, verification_status: bool) {
        self.log_entry_verification_status = verification_status
    }
}
