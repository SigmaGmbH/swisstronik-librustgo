#![no_std]
#![feature(slice_as_chunks)]

#[macro_use]
extern crate sgx_tstd as std;
extern crate rustls;

extern crate sgx_types;
use sgx_types::sgx_status_t;

use internal_types::ExecutionResult;
use protobuf::Message;
use protobuf::RepeatedField;
use sgxvm::primitive_types::{H160, H256, U256};
use sgxvm::{self, Vicinity};
// use std::panic::catch_unwind;
// use std::ptr;
use std::slice;
use std::vec::Vec;

// use crate::error::{handle_c_error_default, Error};
// use crate::memory::{ByteSliceView, UnmanagedVector};
use crate::protobuf_generated::ffi::{
    AccessListItem, FFIRequest, FFIRequest_oneof_req, HandleTransactionResponse, Log,
    SGXVMCallRequest, SGXVMCreateRequest, Topic, TransactionContext as ProtoTransactionContext, NodePublicKeyResponse,
};
use crate::querier::GoQuerier;

mod backend;
mod coder;
mod error;
mod memory;
mod ocall;
mod protobuf_generated;
mod querier;
mod storage;
mod encryption;
mod attestation;
mod key_manager;
mod handlers;

pub const MAX_RESULT_LEN: usize = 4096;

#[repr(C)]
pub struct AllocationWithResult {
    pub result_ptr: *mut u8,
    pub result_len: usize,
    pub status: sgx_status_t
}

impl Default for AllocationWithResult {
    fn default() -> Self {
        AllocationWithResult {
            result_ptr: std::ptr::null_mut(),
            result_len: 0,
            status: sgx_status_t::SGX_ERROR_UNEXPECTED,
        }
    }
}

#[repr(C)]
pub struct Allocation {
    pub result_ptr: *mut u8,
    pub result_size: usize,
}

#[no_mangle]
/// Checks if there is already sealed master key
pub unsafe extern "C" fn ecall_is_initialized() -> i32 {
    if let Err(err) = key_manager::KeyManager::unseal() {
        println!("[Enclave] Cannot restore master key. Reason: {:?}", err.as_str());
        return false as i32
    }
    true as i32
} 

#[no_mangle]
pub extern "C" fn ecall_allocate(
    data: *const u8,
    len: usize,
) -> Allocation {
    // TODO: In case of any errors check: https://github.com/scrtlabs/SecretNetwork/blob/8e157399de55c8e9c3f9a05d2d23e259dae24095/cosmwasm/enclaves/shared/contract-engine/src/external/ecalls.rs#L41
    let slice = unsafe { slice::from_raw_parts(data, len) };
    let mut vector_copy = slice.to_vec();

    let ptr = vector_copy.as_mut_ptr();
    let size = vector_copy.len();
    std::mem::forget(vector_copy); // TODO: Need to clean that memory

    Allocation { result_ptr: ptr, result_size: size }
}

#[no_mangle]
/// Handles incoming protobuf-encoded request
pub extern "C" fn handle_request(
    querier: *mut GoQuerier,
    request_data: *const u8,
    len: usize,
) -> AllocationWithResult {
    let request_slice = unsafe { slice::from_raw_parts(request_data, len) };

    let ffi_request = match protobuf::parse_from_bytes::<FFIRequest>(request_slice) {
        Ok(ffi_request) => ffi_request,
        Err(err) => {
            println!("Got error during protobuf decoding: {:?}", err);
            return AllocationWithResult::default();
        }
    };

    match ffi_request.req {
        Some(req) => {
            match req {
                FFIRequest_oneof_req::callRequest(data) => {
                    handlers::tx::handle_call_request(querier, data)
                },
                FFIRequest_oneof_req::createRequest(data) => {
                    handlers::tx::handle_create_request(querier, data)
                },
                FFIRequest_oneof_req::publicKeyRequest(_) => {
                    let res = encryption::x25519_get_public_key();
                    match res {
                        Ok(res) => {
                            let mut response = NodePublicKeyResponse::new();
                            response.set_publicKey(res);

                            let encoded_response = match response.write_to_bytes() {
                                Ok(res) => res,
                                Err(err) => {
                                    println!("Cannot encode protobuf result");
                                    return AllocationWithResult::default();
                                }
                            };
                            
                            handlers::allocate_inner(encoded_response)
                        },
                        Err(err) => {
                            println!("Cannot obtain node public key. Reason: {:?}", err);
                            return AllocationWithResult::default();
                        }
                    }
                }
            }
        }
        None => {
            println!("Got empty request during protobuf decoding");
            AllocationWithResult::default()
        }
    }
}
