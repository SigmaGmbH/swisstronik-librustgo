use cosmwasm_vm::VmError;
use errno::{set_errno, Errno};
#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use thiserror::Error;

use crate::memory::Buffer;

#[derive(Error, Debug)]
pub enum RustError {
    #[error("Null/Empty argument: {}", name)]
    EmptyArg {
        name: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[error("Cannot decode UTF8 bytes into string: {}", msg)]
    InvalidUtf8 {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Ran out of gas")]
    OutOfGas {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Caught panic")]
    Panic {
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error calling the VM: {}", msg)]
    VmErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl RustError {
    pub fn empty_arg<T: Into<String>>(name: T) -> Self {
        RustError::EmptyArg {
            name: name.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_utf8<S: ToString>(msg: S) -> Self {
        RustError::InvalidUtf8 {
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn panic() -> Self {
        RustError::Panic {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn vm_err<S: ToString>(msg: S) -> Self {
        RustError::VmErr {
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn out_of_gas() -> Self {
        RustError::OutOfGas {
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl From<VmError> for RustError {
    fn from(source: VmError) -> Self {
        match source {
            VmError::GasDepletion { .. } => RustError::out_of_gas(),
            _ => RustError::vm_err(source),
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

/// cbindgen:prefix-with-name
#[repr(i32)]
enum ErrnoValue {
    Success = 0,
    Other = 1,
    OutOfGas = 2,
}

pub fn clear_error() {
    set_errno(Errno(ErrnoValue::Success as i32));
}

pub fn set_error(err: RustError, errout: Option<&mut Buffer>) {
    let msg = err.to_string();
    if let Some(mb) = errout {
        *mb = Buffer::from_vec(msg.into_bytes());
    }
    let errno = match err {
        RustError::OutOfGas { .. } => ErrnoValue::OutOfGas,
        _ => ErrnoValue::Other,
    } as i32;
    set_errno(Errno(errno));
}

/// If `result` is Ok, this returns the binary representation of the Ok value and clears [errno].
/// Otherwise it returns an empty vector, writes the error message to `errout` and sets [errno].
///
/// [errno]: https://utcc.utoronto.ca/~cks/space/blog/programming/GoCgoErrorReturns
pub fn handle_c_error_binary<T>(
    result: Result<T, RustError>,
    errout: Option<&mut Buffer>,
) -> Vec<u8>
where
    T: Into<Vec<u8>>,
{
    match result {
        Ok(value) => {
            clear_error();
            value.into()
        }
        Err(error) => {
            set_error(error, errout);
            Vec::new()
        }
    }
}

/// If `result` is Ok, this returns the Ok value and clears [errno].
/// Otherwise it returns the default value, writes the error message to `errout` and sets [errno].
///
/// [errno]: https://utcc.utoronto.ca/~cks/space/blog/programming/GoCgoErrorReturns
pub fn handle_c_error_default<T>(result: Result<T, RustError>, errout: Option<&mut Buffer>) -> T
where
    T: Default,
{
    match result {
        Ok(value) => {
            clear_error();
            value
        }
        Err(error) => {
            set_error(error, errout);
            Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_vm::{BackendError, Checksum};
    use errno::errno;
    use std::str;

    #[test]
    fn empty_arg_works() {
        let error = RustError::empty_arg("gas");
        match error {
            RustError::EmptyArg { name, .. } => {
                assert_eq!(name, "gas");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_strings() {
        let error = RustError::invalid_utf8("my text");
        match error {
            RustError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_errors() {
        let original = String::from_utf8(vec![0x80]).unwrap_err();
        let error = RustError::invalid_utf8(original);
        match error {
            RustError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 1 bytes from index 0");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn panic_works() {
        let error = RustError::panic();
        match error {
            RustError::Panic { .. } => {}
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn vm_err_works_for_strings() {
        let error = RustError::vm_err("my text");
        match error {
            RustError::VmErr { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn vm_err_works_for_errors() {
        // No public interface exists to generate a VmError directly
        let original: VmError = BackendError::out_of_gas().into();
        let error = RustError::vm_err(original);
        match error {
            RustError::VmErr { msg, .. } => {
                assert_eq!(msg, "Ran out of gas during contract execution");
            }
            _ => panic!("expect different error"),
        }
    }

    // Tests of `impl From<X> for RustError` converters

    #[test]
    fn from_std_str_utf8error_works() {
        let error: RustError = str::from_utf8(b"Hello \xF0\x90\x80World")
            .unwrap_err()
            .into();
        match error {
            RustError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn from_std_string_fromutf8error_works() {
        let error: RustError = String::from_utf8(b"Hello \xF0\x90\x80World".to_vec())
            .unwrap_err()
            .into();
        match error {
            RustError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn handle_c_error_binary_works() {
        let mut error_msg = Buffer::default();

        // Ok (non-empty vector)
        let res: Result<Vec<u8>, RustError> = Ok(vec![0xF0, 0x0B, 0xAA]);
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, vec![0xF0, 0x0B, 0xAA]);

        // Ok (empty vector)
        let res: Result<Vec<u8>, RustError> = Ok(vec![]);
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Ok (non-empty slice)
        let res: Result<&[u8], RustError> = Ok(b"foobar");
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::from(b"foobar" as &[u8]));

        // Ok (empty slice)
        let res: Result<&[u8], RustError> = Ok(b"");
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Ok (checksum)
        let res: Result<Checksum, RustError> = Ok(Checksum::from([
            0x72, 0x2c, 0x8c, 0x99, 0x3f, 0xd7, 0x5a, 0x76, 0x27, 0xd6, 0x9e, 0xd9, 0x41, 0x34,
            0x4f, 0xe2, 0xa1, 0x42, 0x3a, 0x3e, 0x75, 0xef, 0xd3, 0xe6, 0x77, 0x8a, 0x14, 0x28,
            0x84, 0x22, 0x71, 0x04,
        ]));
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(
            data,
            vec![
                0x72, 0x2c, 0x8c, 0x99, 0x3f, 0xd7, 0x5a, 0x76, 0x27, 0xd6, 0x9e, 0xd9, 0x41, 0x34,
                0x4f, 0xe2, 0xa1, 0x42, 0x3a, 0x3e, 0x75, 0xef, 0xd3, 0xe6, 0x77, 0x8a, 0x14, 0x28,
                0x84, 0x22, 0x71, 0x04,
            ]
        );

        // Err (vector)
        let res: Result<Vec<u8>, RustError> = Err(RustError::panic());
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Err (slice)
        let res: Result<&[u8], RustError> = Err(RustError::panic());
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Err (checksum)
        let res: Result<Checksum, RustError> = Err(RustError::panic());
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());
    }

    #[test]
    fn handle_c_error_binary_clears_an_old_error() {
        let mut error_msg = Buffer::default();

        // Err
        let res: Result<Vec<u8>, RustError> = Err(RustError::panic());
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Ok
        let res: Result<Vec<u8>, RustError> = Ok(vec![0xF0, 0x0B, 0xAA]);
        let data = handle_c_error_binary(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_ne!(error_msg.len, 0); // Message is not reset
        assert_eq!(data, vec![0xF0, 0x0B, 0xAA]);
    }

    #[test]
    fn handle_c_error_default_works() {
        let mut error_msg = Buffer::default();

        // Ok (non-empty vector)
        let res: Result<Vec<u8>, RustError> = Ok(vec![0xF0, 0x0B, 0xAA]);
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, vec![0xF0, 0x0B, 0xAA]);

        // Ok (empty vector)
        let res: Result<Vec<u8>, RustError> = Ok(vec![]);
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Ok (non-empty slice)
        let res: Result<&[u8], RustError> = Ok(b"foobar");
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::from(b"foobar" as &[u8]));

        // Ok (empty slice)
        let res: Result<&[u8], RustError> = Ok(b"");
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Ok (unit)
        let res: Result<(), RustError> = Ok(());
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_eq!(error_msg.len, 0);
        assert_eq!(data, ());

        // Err (vector)
        let res: Result<Vec<u8>, RustError> = Err(RustError::panic());
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Err (slice)
        let res: Result<&[u8], RustError> = Err(RustError::panic());
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Err (unit)
        let res: Result<(), RustError> = Err(RustError::panic());
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, ());
    }

    #[test]
    fn handle_c_error_default_clears_an_old_error() {
        let mut error_msg = Buffer::default();

        // Err
        let res: Result<Vec<u8>, RustError> = Err(RustError::panic());
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Other as i32);
        assert_ne!(error_msg.len, 0);
        assert_eq!(data, Vec::<u8>::new());

        // Ok
        let res: Result<Vec<u8>, RustError> = Ok(vec![0xF0, 0x0B, 0xAA]);
        let data = handle_c_error_default(res, Some(&mut error_msg));
        assert_eq!(errno().0, ErrnoValue::Success as i32);
        assert_ne!(error_msg.len, 0); // Message is not reset
        assert_eq!(data, vec![0xF0, 0x0B, 0xAA]);
    }
}
