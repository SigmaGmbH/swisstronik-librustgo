use sgx_types::sgx_quote_sign_type_t;

pub const MRSIGNER: [u8; 32] = [131, 215, 25, 231, 125, 234, 202, 20, 112, 246, 186, 246, 42, 77, 119, 67, 3, 200, 153, 219, 105, 2, 15, 156, 112, 238, 29, 252, 8, 199, 206, 158];

pub const DEV_HOSTNAME: &'static str = "api.trustedservices.intel.com";
pub const SIGRL_SUFFIX: &'static str = "/sgx/dev/attestation/v4/sigrl/";
pub const REPORT_SUFFIX: &'static str = "/sgx/dev/attestation/v4/report";
pub const CERTEXPIRYDAYS: i64 = 90i64;

pub const PUBLIC_KEY_SIZE: usize = 32;
pub const ENCRYPTED_KEY_SIZE: usize = 78;

pub const QUOTE_SIGNATURE_TYPE: sgx_quote_sign_type_t = sgx_quote_sign_type_t::SGX_LINKABLE_SIGNATURE; 