use sgx_types::sgx_status_t;
use sgxvm::evm::backend::Basic;
use sgxvm::primitive_types::{H160, H256, U256};
use sgxvm::storage::Storage;
use std::vec::Vec;

use crate::protobuf_generated::ffi;
use crate::querier::GoQuerier;
use crate::ocall;
use crate::coder;

/// This struct allows us to obtain state from keeper
/// that is located outside of Rust code
pub struct FFIStorage {
    pub querier: *mut GoQuerier,
}

impl Storage for FFIStorage {
    fn contains_key(&self, key: &H160) -> bool {
        let encoded_request = coder::encode_contains_key(key);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            // Decode protobuf
            let decoded_result = match protobuf::parse_from_bytes::<ffi::QueryContainsKeyResponse>(result.as_slice()) {
                Ok(res) => res,
                Err(err) => {
                    println!("Cannot decode protobuf response: {:?}", err);
                    return false
                }
            };
            return decoded_result.contains;
        } else {
            println!("Contains key failed. Empty response");
            return false;
        };
    }

    fn get_account_storage_cell(&self, key: &H160, index: &H256) -> Option<H256> {
        let encoded_request = coder::encode_get_storage_cell(key, index);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            // Decode protobuf
            let decoded_result = match protobuf::parse_from_bytes::<ffi::QueryGetAccountStorageCellResponse>(result.as_slice()) {
                Ok(res) => res,
                Err(err) => {
                    println!("Cannot decode protobuf response: {:?}", err);
                    return None
                }
            };
            return Some(H256::from_slice(decoded_result.value.as_slice()));
        } else {
            println!("Get account storage cell failed. Empty response");
            return None
        }
    }

    fn get_account_code(&self, key: &H160) -> Option<Vec<u8>> {
        let encoded_request = coder::encode_get_account_code(key);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            // Decode protobuf
            let decoded_result = match protobuf::parse_from_bytes::<ffi::QueryGetAccountCodeResponse>(result.as_slice()) {
                Ok(res) => res,
                Err(err) => {
                    println!("Cannot decode protobuf response: {:?}", err);
                    return None
                }
            };
            return Some(decoded_result.code);
        } else {
            println!("Get account code failed. Empty response");
            return None
        }
    }

    fn get_account(&self, key: &H160) -> Basic {
        let encoded_request = coder::encode_get_account(key);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            // Decode protobuf
            let decoded_result = match protobuf::parse_from_bytes::<ffi::QueryGetAccountResponse>(result.as_slice()) {
                Ok(res) => res,
                Err(err) => {
                    println!("Cannot decode protobuf response: {:?}", err);
                    return Basic {
                        balance: U256::default(),
                        nonce: U256::default(),
                    };
                }
            };
            return Basic {
                balance: U256::from_big_endian(decoded_result.balance.as_slice()),
                nonce: U256::from(decoded_result.nonce),
            };
        } else {
            println!("Get account failed. Empty response");
            return Basic {
                balance: U256::default(),
                nonce: U256::default(),
            };
        }
    }

    fn insert_account(&mut self, key: H160, data: Basic) {
        let encoded_request = coder::encode_insert_account(key, data);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            match protobuf::parse_from_bytes::<ffi::QueryInsertAccountResponse>(result.as_slice()) {
                Err(err) => {
                    println!("Cannot decode protobuf. Got error: {:?}", err);
                },
                _ => {}
            }
        } else {
            println!("Insert account failed. Empty response");
        }
    }

    fn insert_account_code(&mut self, key: H160, code: Vec<u8>) {
        let encoded_request = coder::encode_insert_account_code(key, code);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            match protobuf::parse_from_bytes::<ffi::QueryInsertAccountCodeResponse>(result.as_slice()) {
                Err(err) => {
                    println!("Cannot decode protobuf. Got error: {:?}", err);
                },
                _ => {}
            }
        } else {
            println!("Insert account code failed. Empty response");
        }
    }

    fn insert_storage_cell(&mut self, key: H160, index: H256, value: H256) {
        let encoded_request = coder::encode_insert_storage_cell(key, index, value);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            match protobuf::parse_from_bytes::<ffi::QueryInsertStorageCellResponse>(result.as_slice()) {
                Err(err) => {
                    println!("Cannot decode protobuf. Got error: {:?}", err);
                },
                _ => {}
            }
        } else {
            println!("Insert storage cell failed. Empty response");
        }
    }

    fn remove(&mut self, key: &H160) {
        let encoded_request = coder::encode_remove(key);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            match protobuf::parse_from_bytes::<ffi::QueryRemoveResponse>(result.as_slice()) {
                Err(err) => {
                    println!("Cannot decode protobuf. Got error: {:?}", err);
                },
                _ => {}
            }
        } else {
            println!("Remove failed. Empty response");
        }
    }

    fn remove_storage_cell(&mut self, key: &H160, index: &H256) {
        let encoded_request = coder::encode_remove_storage_cell(key, index);
        if let Some(result) = ocall::make_request(self.querier, encoded_request) {
            match protobuf::parse_from_bytes::<ffi::QueryRemoveStorageCellResponse>(result.as_slice()) {
                Err(err) => {
                    println!("Cannot decode protobuf. Got error: {:?}", err);
                },
                _ => {}
            }
        } else {
            println!("Remove storage cell failed. Empty response");
        }
    }
}

impl FFIStorage {
    pub fn new(querier: *mut GoQuerier) -> Self {
        Self {querier}
    }
}
