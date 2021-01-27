package api

/*
#include "bindings.h"
#include <stdio.h>

// imports (db)
GoResult cSet(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, Buffer *errOut);
GoResult cGet(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, Buffer *val, Buffer *errOut);
GoResult cDelete(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, Buffer *errOut);
GoResult cScan(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, Buffer *errOut);
// imports (iterator)
GoResult cNext(iterator_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, Buffer *key, Buffer *val, Buffer *errOut);
// imports (api)
GoResult cHumanAddress(api_t *ptr, U8SliceView src, UnmanagedVector *dest, Buffer *errOut, uint64_t *used_gas);
GoResult cCanonicalAddress(api_t *ptr, U8SliceView src, UnmanagedVector *dest, Buffer *errOut, uint64_t *used_gas);
// imports (querier)
GoResult cQueryExternal(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, Buffer *result, Buffer *errOut);

// Gateway functions (db)
GoResult cGet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, Buffer *val, Buffer *errOut) {
	return cGet(ptr, gas_meter, used_gas, key, val, errOut);
}
GoResult cSet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, Buffer *errOut) {
	return cSet(ptr, gas_meter, used_gas, key, val, errOut);
}
GoResult cDelete_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, Buffer *errOut) {
	return cDelete(ptr, gas_meter, used_gas, key, errOut);
}
GoResult cScan_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, Buffer *errOut) {
	return cScan(ptr, gas_meter, used_gas, start, end, order, out, errOut);
}

// Gateway functions (iterator)
GoResult cNext_cgo(iterator_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, Buffer *key, Buffer *val, Buffer *errOut) {
	return cNext(ptr, gas_meter, used_gas, key, val, errOut);
}

// Gateway functions (api)
GoResult cCanonicalAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, Buffer *errOut, uint64_t *used_gas) {
    return cCanonicalAddress(ptr, src, dest, errOut, used_gas);
}
GoResult cHumanAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, Buffer *errOut, uint64_t *used_gas) {
    return cHumanAddress(ptr, src, dest, errOut, used_gas);
}

// Gateway functions (querier)
GoResult cQueryExternal_cgo(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, Buffer *result, Buffer *errOut) {
    return cQueryExternal(ptr, gas_limit, used_gas, request, result, errOut);
}
*/
import "C"

// We need these gateway functions to allow calling back to a go function from the c code.
// At least I didn't discover a cleaner way.
// Also, this needs to be in a different file than `callbacks.go`, as we cannot create functions
// in the same file that has //export directives. Only import header types
