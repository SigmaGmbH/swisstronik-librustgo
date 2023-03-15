SHELL := /bin/bash
COMPILER = rustc
TARGET = $(shell rustc --version --verbose 2> /dev/null | awk "/host:/ { print \$$2 }")
TARGET_DIR = target/release/
DEFAULT = help

define sgx_clean
	@echo "Cleaning enclave dependencies"
	@rm -rf ./bin/*
	@rm -rf ./lib/*
	@rm -f ./Enclave_u*
	@rm -rf ./sgx_evm/target
	@rm -f ./sgx_evm/Enclave_t*
	@rm -f ./sgx_evm/enclave.unsigned.so
endef

define sgx_build
	$(call sgx_clean)
	@echo "Building enclave"
	@CARGO_TARGET_DIR=./sgx_evm/target RUSTFLAGS="-C target-cpu=native" cargo build --release --manifest-path ./sgx_evm/Cargo.toml
	@/opt/intel/sgxsdk/bin/x64/sgx_edger8r --trusted ./sgx_evm/Enclave.edl --search-path /opt/intel/sgxsdk/include --search-path ./sdk/edl --trusted-dir ./sgx_evm
	@/opt/intel/sgxsdk/bin/x64/sgx_edger8r --untrusted ./sgx_evm/Enclave.edl --search-path /opt/intel/sgxsdk/include --search-path ./sdk/edl --untrusted-dir ./
	@cc -m64 -O2 -fstack-protector -fPIC -Wno-attributes -I ./ -I./include -I/opt/intel/sgxsdk/include -I./sdk/edl -c ./Enclave_u.c -o ./Enclave_u.o
	@ar rcsD ./lib/libEnclave_u.a ./Enclave_u.o
	@cp ./sgx_evm/target/release/libenclave.a ./lib/libenclave.a
	@cc -m64 -O2 -fstack-protector -ffreestanding -nostdinc -fvisibility=hidden -fpie -fno-strict-overflow -fno-delete-null-pointer-checks -I./sdk/common/inc -I./sdk/edl -I/opt/intel/sgxsdk/include -I/opt/intel/sgxsdk/include/tlibc -I/opt/intel/sgxsdk/include/stlport -I/opt/intel/sgxsdk/include/epid -I ./enclave -I./include -c ./enclave/Enclave_t.c -o ./enclave/Enclave_t.o
	@g++ ./sgx_evm/Enclave_t.o -o ./sgx_evm/enclave.unsigned.so -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles -L/opt/intel/sgxsdk/lib64 -Wl,--whole-archive -lsgx_trts -Wl,--no-whole-archive -Wl,--start-group -lsgx_tstdc -lsgx_tcxx -lsgx_tservice -lsgx_tcrypto -lsgx_urts -lpthread -L./lib -lenclave -Wl,--end-group -Wl,--version-script=./sgx_evm/Enclave.lds -Wl,-z,relro,-z,now,-z,noexecstack -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined -Wl,-pie,-eenclave_entry -Wl,--export-dynamic -Wl,--gc-sections -Wl,--defsym,__ImageBase=0
	@echo "Signing enclave"
	@/opt/intel/sgxsdk/bin/x64/sgx_sign sign -key ./sgx_evm/Enclave_private.pem -enclave ./sgx_evm/enclave.unsigned.so -out ./bin/enclave.signed.so -config ./sgx_evm/Enclave.config.xml
endef

all: $(DEFAULT)

help:
	@echo "Usage:"
	@echo "	make build						- Builds application"
	@echo "	make sgx"						- Builds Intel SGX enclave

.PHONY: \
		build \
		sgx \
		clean \

clean:
	$(call dcap_clean)
	$(call sgx_clean)
	cargo clean
	rm -f "$(TARGET_DIR)"
	echo "Binaries and dependencies deleted"

build:
	$(call sgx_build)
	@echo "Build application"
	@cargo build --release
	@cp ./bin/enclave.signed.so ./target/release/enclave.signed.so
	@echo "Application built"

sgx:
	$(call sgx_build)
	@echo "Intel SGX enclave built and signed"