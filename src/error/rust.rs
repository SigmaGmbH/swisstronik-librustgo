use errno::{set_errno, Errno};
use std::fmt;

use cosmwasm_vm::VmError;
use snafu::Snafu;

use crate::memory::Buffer;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Wasm Error: {}", source))]
    WasmErr {
        source: VmError,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Caught Panic"))]
    Panic {
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Null/Empty argument passed: {}", name))]
    EmptyArg {
        name: &'static str,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
    #[snafu(display("Invalid string format: {}", source))]
    Utf8Err {
        source: std::str::Utf8Error,
        #[cfg(feature = "backtraces")]
        backtrace: snafu::Backtrace,
    },
}

/// empty_err returns an error with stack trace.
/// helper to construct Error::EmptyArg  over and over.
pub(crate) fn empty_err(name: &'static str) -> Error {
    EmptyArg { name }.build()
}

pub fn clear_error() {
    set_errno(Errno(0));
}

pub fn set_error(msg: String, errout: Option<&mut Buffer>) {
    if let Some(mb) = errout {
        *mb = Buffer::from_vec(msg.into_bytes());
    }
    // Question: should we set errno to something besides generic 1 always?
    set_errno(Errno(1));
}

pub fn handle_c_error<T, E>(r: Result<T, E>, errout: Option<&mut Buffer>) -> T
where
    T: Default,
    E: fmt::Display,
{
    match r {
        Ok(t) => {
            clear_error();
            t
        }
        Err(e) => {
            set_error(e.to_string(), errout);
            T::default()
        }
    }
}
