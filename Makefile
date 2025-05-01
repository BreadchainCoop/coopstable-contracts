SUBDIRS := contracts/cusd_manager contracts/lending_yield_controller contracts/yield_adapter_registry packages/blend_capital_adapter packages/yield_adapter
BUILD_FLAGS ?=

default: build
all: test

build:
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir build WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

test: build
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir test WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

fmt:
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir fmt WORKSPACE_ROOT=$(PWD) || exit 1; \
	done

clean:
	@for dir in $(SUBDIRS) ; do \
		$(MAKE) -C $$dir clean WORKSPACE_ROOT=$(PWD) || exit 1; \
	done