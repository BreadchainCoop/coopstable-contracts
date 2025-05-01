WORKSPACE_ROOT ?= ../..

default: build
all: test
test: build
	cargo test
build:
	stellar contract build
	@ls -l $(WORKSPACE_ROOT)/target/wasm32-unknown-unknown/release/*.wasm
fmt:
	cargo fmt --all
clean:
	cargo clean