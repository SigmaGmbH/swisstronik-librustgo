/* (c) 2022 Sigma Assets GmbH. Licensed under Apache-2.0 */

/* Generated with cbindgen:0.24.3 */

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

enum ErrnoValue {
  ErrnoValue_Success = 0,
  ErrnoValue_Other = 1,
  ErrnoValue_OutOfGas = 2,
};
typedef int32_t ErrnoValue;

/**
 * This enum gives names to the status codes returned from Go callbacks to Rust.
 * The Go code will return one of these variants when returning.
 *
 * 0 means no error, all the other cases are some sort of error.
 *
 */
enum GoError {
  GoError_None = 0,
  /**
   * Go panicked for an unexpected reason.
   */
  GoError_Panic = 1,
  /**
   * Go received a bad argument from Rust
   */
  GoError_BadArgument = 2,
  /**
   * Ran out of gas while using the SDK (e.g. storage). This can come from the Cosmos SDK gas meter
   * (https://github.com/cosmos/cosmos-sdk/blob/v0.45.4/store/types/gas.go#L29-L32).
   */
  GoError_OutOfGas = 3,
  /**
   * Error while trying to serialize data in Go code (typically json.Marshal)
   */
  GoError_CannotSerialize = 4,
  /**
   * An error happened during normal operation of a Go callback, which should be fed back to the contract
   */
  GoError_User = 5,
  /**
   * An error happend during interacting with DataQuerier (failed to apply some changes / failed to create contract / etc. )
   */
  GoError_QuerierError = 6,
  /**
   * An error type that should never be created by us. It only serves as a fallback for the i32 to GoError conversion.
   */
  GoError_Other = -1,
};
typedef int32_t GoError;

typedef struct Vec_u8 Vec_u8;

extern struct Vec_u8 get_block_hash(struct Vec_u8 req);

extern struct Vec_u8 get_account(struct Vec_u8 req);

extern struct Vec_u8 contains_key(struct Vec_u8 req);

extern struct Vec_u8 get_storage_cell(struct Vec_u8 req);

extern struct Vec_u8 get_account_code(struct Vec_u8 req);

extern struct Vec_u8 insert_account(struct Vec_u8 req);

extern struct Vec_u8 insert_account_code(struct Vec_u8 req);

extern struct Vec_u8 insert_storage_cell(struct Vec_u8 req);

extern struct Vec_u8 remove(struct Vec_u8 req);

extern struct Vec_u8 remove_storage_cell(struct Vec_u8 req);
