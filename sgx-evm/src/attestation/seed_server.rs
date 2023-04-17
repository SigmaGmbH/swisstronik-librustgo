use sgx_tcrypto::*;
use sgx_types::*;

use rustls;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::prelude::v1::*;
use std::str;
use std::sync::Arc;
use std::vec::Vec;

use super::consts::QUOTE_SIGNATURE_TYPE;

#[no_mangle]
pub unsafe extern "C" fn ecall_share_seed(socket_fd: c_int) -> sgx_status_t {
    share_seed_inner(socket_fd)
}

#[cfg(feature = "hardware_mode")]
fn share_seed_inner(socket_fd: c_int) -> sgx_status_t {
    let cfg = match get_server_configuration() {
        Ok(cfg) => cfg,
        Err(err) => {
            println!("{}", err);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut sess = rustls::ServerSession::new(&Arc::new(cfg));
    let mut conn = match TcpStream::new(socket_fd) {
        Ok(conn) => conn,
        Err(err) => {
            println!(
                "[Enclave] Seed Server: cannot establish connection with client: {:?}",
                err
            );
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut tls = rustls::Stream::new(&mut sess, &mut conn);
    let mut plaintext = Vec::new();
    if let Err(err) = tls.read(&mut plaintext) {
        println!("[Enclave] Seed Server: error in read_to_end: {:?}", err);
        return sgx_status_t::SGX_ERROR_UNEXPECTED;
    };

    // TODO: Unseal or make key manager static
    // TODO: Add encryption

    tls.write("hello back".as_bytes()).unwrap();

    sgx_status_t::SGX_SUCCESS
}

#[cfg(not(feature = "hardware_mode"))]
fn share_seed_inner(socket_fd: c_int) -> sgx_status_t {
    let mut conn = match TcpStream::new(socket_fd) {
        Ok(conn) => conn,
        Err(err) => {
            println!(
                "[Enclave] Seed Server: cannot establish connection with client: {:?}",
                err
            );
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    let mut plaintext = [0u8; 1024]; //Vec::new();
    match conn.read(&mut plaintext) {
        Ok(_) => {
            /*
               TODO:
               1. Get public key from client
               2. Create encryption key
               3. Encrypt seed
               4. Send to client
            */
            println!("Client said: {}", str::from_utf8(&plaintext).unwrap())
        }
        Err(e) => {
            println!("Error in read_to_end: {:?}", e);
            return sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    };

    conn.write("hello back".as_bytes()).unwrap();
    sgx_status_t::SGX_SUCCESS
}

#[cfg(feature = "hardware_mode")]
fn get_server_configuration() -> Result<rustls::ServerConfig, String> {
    // Generate Keypair
    let ecc_handle = SgxEccHandle::new();
    let _result = ecc_handle.open();
    let (prv_k, pub_k) = ecc_handle.create_key_pair().unwrap();

    let signed_report = match super::utils::create_attestation_report(&pub_k, QUOTE_SIGNATURE_TYPE)
    {
        Ok(r) => r,
        Err(e) => {
            return Err(format!("Error creating attestation report"));
        }
    };

    let payload: String = match serde_json::to_string(&signed_report) {
        Ok(payload) => payload,
        Err(err) => {
            return Err(format!(
                "Error serializing report. May be malformed, or badly encoded: {:?}",
                err
            ));
        }
    };
    let (key_der, cert_der) = match super::cert::gen_ecc_cert(payload, &prv_k, &pub_k, &ecc_handle)
    {
        Ok(r) => r,
        Err(e) => {
            return Err(format!("Error in gen_ecc_cert: {:?}", e));
        }
    };
    let _result = ecc_handle.close();

    let mut cfg = rustls::ServerConfig::new(Arc::new(super::utils::ClientAuth::new(true)));
    let mut certs = Vec::new();
    certs.push(rustls::Certificate(cert_der));
    let privkey = rustls::PrivateKey(key_der);

    cfg.set_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![])
        .unwrap();

    Ok(cfg)
}
