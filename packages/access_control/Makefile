WORKSPACE_ROOT ?= ../..

default: build
all: test
test: build
	cargo test
build:
	stellar contract build
	@ls -l $(WORKSPACE_ROOT)/target/wasm32v1-none/release/*.wasm
fmt:
	cargo fmt --all
	
clean:
	cargo clean