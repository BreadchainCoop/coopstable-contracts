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

# TODO:
# add automated scripts to deploy to any network
# add automate scripts to generate contract bindings