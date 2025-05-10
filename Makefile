ALL_DIRS := packages/blend_capital_adapter packages/yield_adapter packages/access_control contracts/cusd_manager contracts/yield_adapter_registry contracts/yield_distributor contracts/lending_yield_controller
BINDING_DIRS := packages/yield_adapter contracts/cusd_manager contracts/yield_adapter_registry contracts/yield_distributor contracts/lending_yield_controller

BINDINGS_BASE_DIR := ./ts

BUILD_FLAGS ?=

default: build
all: test

build:
	@for dir in $(ALL_DIRS) ; do \
		$(MAKE) -C $$dir build WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

test: build
	@for dir in $(ALL_DIRS) ; do \
		$(MAKE) -C $$dir test WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

fmt:
	@for dir in $(ALL_DIRS) ; do \
		$(MAKE) -C $$dir fmt WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

clean:
	@for dir in $(ALL_DIRS) ; do \
		$(MAKE) -C $$dir clean WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

RPC_URL := http://localhost:8000
NETWORK_PASSPHRASE := "Standalone Network ; February 2017" 
NETWORK := Standalone
CONTRACT_ID := CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN

generate-ts-bindings: build
	@echo "Generating TypeScript bindings for all contracts and packages..."
	@mkdir -p $(BINDINGS_BASE_DIR)
	@for dir in $(CONTRACT_DIRS) ; do \
		contract_name=$$(basename $$dir); \
		wasm_path=./target/wasm32-unknown-unknown/release/$$contract_name.wasm; \
		output_dir=$(BINDINGS_BASE_DIR)/ts-$$contract_name; \
		mkdir -p $$output_dir; \
		echo "Generating bindings for $$contract_name..."; \
		stellar contract bindings typescript --overwrite \
			--contract-id $(CONTRACT_ID) \
			--wasm $$wasm_path \
			--output-dir $$output_dir \
			--rpc-url $(RPC_URL) \
			--network-passphrase $(NETWORK_PASSPHRASE) \
			--network $(NETWORK) || echo "Failed to generate bindings for $$contract_name"; \
	done
	@echo "TypeScript bindings generation complete!"