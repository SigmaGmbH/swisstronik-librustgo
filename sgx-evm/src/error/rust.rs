use crate::memory::UnmanagedVector;
use errno::{set_errno, Errno};
use std::string::String;
use std::string::ToString;
use std::vec::Vec;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
pub enum RustError {
    #[error("Cannot decode UTF8 bytes into string: {}", msg)]
    InvalidUtf8 { msg: String },
    #[error("Error calling the VM: {}", msg)]
    ProtobufDecodeError { msg: String },
    #[error("Encryption error: {}", msg)]
    EncryptionError { msg: String },
    #[error("Decryption error: {}", msg)]
    DecryptionError { msg: String },
    #[error("Enclave error: {}", msg)]
    EnclaveError { msg: String },
}

impl RustError {
    pub fn invalid_utf8<S: ToString>(msg: S) -> Self {
        RustError::InvalidUtf8 {
            msg: msg.to_string(),
        }
    }

    pub fn encryption_err<S: ToString>(msg: S) -> Self {
        RustError::EncryptionError {
            msg: msg.to_string(),
        }
    }

    pub fn decryption_err<S: ToString>(msg: S) -> Self {
        RustError::DecryptionError {
            msg: msg.to_string(),
        }
    }

    pub fn enclave_err<S: ToString>(msg: S) -> Self {
        RustError::EnclaveError {
            msg: msg.to_string(),
        }
    }
}

impl From<std::str::Utf8Error> for RustError {
    fn from(source: std::str::Utf8Error) -> Self {
        RustError::invalid_utf8(source)
    }
}

impl From<std::string::FromUtf8Error> for RustError {
    fn from(source: std::string::FromUtf8Error) -> Self {
        RustError::invalid_utf8(source)
    }
}