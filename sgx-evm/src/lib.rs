#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate sgx_types;
use sgx_types::sgx_status_t;

use internal_types::ExecutionResult;
use protobuf::Message;
use protobuf::RepeatedField;
use sgxvm::{self, Vicinity};
use sgxvm::primitive_types::{H160, H256, U256};
use std::panic::catch_unwind;
use std::vec::Vec;
use std::slice;

use crate::error::{handle_c_error_default, Error};
use crate::protobuf_generated::ffi::{
    AccessListItem, FFIRequest, FFIRequest_oneof_req, HandleTransactionResponse, Log,
    SGXVMCallRequest, SGXVMCreateRequest, Topic, TransactionContext as ProtoTransactionContext,
};
use crate::memory::{ByteSliceView, UnmanagedVector};
use crate::querier::GoQuerier;

mod error;
mod protobuf_generated;
mod backend;
mod coder;
mod storage;
mod memory;
mod querier;

// store some common string for argument names
pub const PB_REQUEST_ARG: &str = "pb_request";

#[no_mangle]
/// Handles incoming protobuf-encoded request for transaction handling
pub extern "C" fn handle_request(
    querier: *mut GoQuerier,
    request_data: *const u8,
    len: usize,
) -> sgx_types::sgx_status_t {
    let request_slice = unsafe { slice::from_raw_parts(request_data, len) }; 
    println!("hello from enclave. Got request with len: {:?}", request_slice.len());

    let ffi_request = match protobuf::parse_from_bytes::<FFIRequest>(request_slice) {
        Ok(ffi_request) => ffi_request,
        Err(err) => {
            println!("Got error during protobuf decoding: {:?}", err);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    match ffi_request.req {
        Some(req) => {
            match req {
                FFIRequest_oneof_req::callRequest(call_request) => {
                    println!("Got call request");
                },
                FFIRequest_oneof_req::createRequest(create_request) => {
                    println!("Got create request");
                }
            }
        },
        None => {
            println!("Got empty request during protobuf decoding");
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }
    
    sgx_status_t::SGX_SUCCESS
}

fn handle_call_request(querier: *mut GoQuerier, data: SGXVMCallRequest) -> ExecutionResult {
    let params = data.params.unwrap();
    let context = data.context.unwrap();

    let vicinity = Vicinity { origin: H160::from_slice(&params.from) };
    let mut storage = crate::storage::FFIStorage::new(querier);
    let mut backend = backend::FFIBackend::new(
        querier,
        &mut storage,
        vicinity,
        build_transaction_context(context),
    );

    sgxvm::handle_sgxvm_call(
        &mut backend,
        params.gasLimit,
        H160::from_slice(&params.from),
        H160::from_slice(&params.to),
        U256::from_big_endian(&params.value),
        params.data,
        parse_access_list(params.accessList),
        params.commit,
    )
}

fn handle_create_request(querier: *mut GoQuerier, data: SGXVMCreateRequest) -> ExecutionResult {
    let params = data.params.unwrap();
    let context = data.context.unwrap();

    let vicinity = Vicinity { origin: H160::from_slice(&params.from) };
    let mut storage = crate::storage::FFIStorage::new(querier);
    let mut backend = backend::FFIBackend::new(
        querier,
        &mut storage,
        vicinity,
        build_transaction_context(context),
    );

    sgxvm::handle_sgxvm_create(
        &mut backend,
        params.gasLimit,
        H160::from_slice(&params.from),
        U256::from_big_endian(&params.value),
        params.data,
        parse_access_list(params.accessList),
        params.commit,
    )
}

fn parse_access_list(data: RepeatedField<AccessListItem>) -> Vec<(H160, Vec<H256>)> {
    let mut access_list = Vec::default();
    for access_list_item in data.to_vec() {
        let address = H160::from_slice(&access_list_item.address);
        let slots = access_list_item.storageSlot
            .to_vec()
            .into_iter()
            .map(|item| { H256::from_slice(&item) })
            .collect();

        access_list.push((address, slots));
    }

    access_list
}

fn build_transaction_context(context: ProtoTransactionContext) -> backend::TxContext {
    backend::TxContext {
        chain_id: U256::from(context.chain_id),
        gas_price: U256::from_big_endian(&context.gas_price),
        block_number: U256::from(context.block_number),
        timestamp: U256::from(context.timestamp),
        block_gas_limit: U256::from(context.block_gas_limit),
        block_base_fee_per_gas: U256::from_big_endian(&context.block_base_fee_per_gas),
        block_coinbase: H160::from_slice(&context.block_coinbase),
    }
}

fn convert_topic_to_proto(topic: H256) -> Topic {
    let mut protobuf_topic = Topic::new();
    protobuf_topic.set_inner(topic.as_fixed_bytes().to_vec());

    protobuf_topic
}
