use crate::{AllocationWithResult, Allocation};
use crate::protobuf_generated::ffi::{
    AccessListItem, FFIRequest, FFIRequest_oneof_req, HandleTransactionResponse, Log,
    SGXVMCallRequest, SGXVMCreateRequest, Topic, TransactionContext as ProtoTransactionContext, NodePublicKeyResponse,
};
use protobuf::Message;
use sgxvm::primitive_types::{H160, H256, U256};
use std::vec::Vec;
use sgxvm::{self, Vicinity};
use internal_types::ExecutionResult;
use crate::backend;
use crate::GoQuerier;
use protobuf::RepeatedField;

/// Handles incoming request for calling contract or transferring value
pub fn handle_call_request(querier: *mut GoQuerier, data: SGXVMCallRequest) -> AllocationWithResult {
    let res = handle_call_request_inner(querier, data);
    post_transaction_handling(res)
}

/// Handles incoming request for creation of a new contract
pub fn handle_create_request(querier: *mut GoQuerier, data: SGXVMCreateRequest) -> AllocationWithResult {
    let res = handle_create_request_inner(querier, data);
    post_transaction_handling(res)
}

fn post_transaction_handling(execution_result: ExecutionResult) -> AllocationWithResult {
    let mut response = HandleTransactionResponse::new();
    response.set_gas_used(execution_result.gas_used);
    response.set_vm_error(execution_result.vm_error);
    response.set_ret(execution_result.data);

    // Convert logs into proper format
    let converted_logs = execution_result
        .logs
        .into_iter()
        .map(|log| {
            let mut proto_log = Log::new();
            proto_log.set_address(log.address.as_fixed_bytes().to_vec());
            proto_log.set_data(log.data);

            let converted_topics: Vec<Topic> =
                log.topics.into_iter().map(convert_topic_to_proto).collect();
            proto_log.set_topics(converted_topics.into());

            proto_log
        })
        .collect();

    response.set_logs(converted_logs);

    let encoded_response = match response.write_to_bytes() {
        Ok(res) => res,
        Err(err) => {
            println!("Cannot encode protobuf result");
            return AllocationWithResult::default();
        }
    };
    
    super::allocate_inner(encoded_response)
}

fn handle_call_request_inner(querier: *mut GoQuerier, data: SGXVMCallRequest) -> ExecutionResult {
    let params = data.params.unwrap();
    let context = data.context.unwrap();

    let vicinity = Vicinity {
        origin: H160::from_slice(&params.from),
    };
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

fn handle_create_request_inner(querier: *mut GoQuerier, data: SGXVMCreateRequest) -> ExecutionResult {
    let params = data.params.unwrap();
    let context = data.context.unwrap();

    let vicinity = Vicinity {
        origin: H160::from_slice(&params.from),
    };
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
        let slots = access_list_item
            .storageSlot
            .to_vec()
            .into_iter()
            .map(|item| H256::from_slice(&item))
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