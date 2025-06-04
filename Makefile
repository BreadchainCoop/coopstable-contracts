# Coopstable Protocol Makefile - Build and Deploy
# This Makefile handles both building and deployment of the Coopstable protocol

# Build configuration
ALL_DIRS := packages/blend_capital_adapter packages/yield_adapter packages/access_control contracts/cusd_manager contracts/yield_adapter_registry contracts/yield_distributor contracts/lending_yield_controller
CONTRACTS := cusd_manager yield_adapter_registry yield_distributor lending_yield_controller blend_capital_adapter

BINDINGS_BASE_DIR := ./ts
BUILD_FLAGS ?=

# Deployment configuration
NETWORK ?= testnet
WASM_DIR = ./target/wasm32v1-none/release

# Account keys (set these as environment variables or override them)
OWNER_KEY ?= owner
ADMIN_KEY ?= admin
TREASURY_KEY ?= treasury

# Get public keys from stellar keys
OWNER := $(shell stellar keys public-key $(OWNER_KEY))
ADMIN := $(shell stellar keys public-key $(ADMIN_KEY))
TREASURY := $(shell stellar keys public-key $(TREASURY_KEY))

# Contract IDs from testnet.json (v2)
CUSD_ID = CDHHR356G725HNLAAQ74WBGVT6Y6ZFZLM2TIHLDCOZTJ2SVZ7P3EANYT
BLEND_POOL_ID = CAMKTT6LIXNOKZJVFI64EBEQE25UYAQZBTHDIQ4LEDJLTCM6YVME6IIY
BLEND_TOKEN_ID = CB22KRA3YZVCNCQI64JQ5WE7UY2VAV7WFLK6A2JN3HEX56T2EDAFO7QF
USDC_ID = CAQCFVLOBK5GIULPNZRGATJJMIZL5BSP7X5YJVMGCPTUEPFM4AVSRCJU

# Contract addresses (will be set after deployment or loaded from file)
-include deployed_addresses.mk

# Default values
TREASURY_SHARE_BPS ?= 1000
DISTRIBUTION_PERIOD ?= 60

# Colors for output - using printf for proper color rendering
GREEN := \033[0;32m
YELLOW := \033[0;33m
RED := \033[0;31m
NC := \033[0m

# Default target
default: build

# All target
all: test

# Help target
.PHONY: help
help:
	@printf "$(GREEN)Coopstable Protocol Makefile$(NC)\n"
	@printf "\n"
	@printf "$(YELLOW)Build targets:$(NC)\n"
	@printf "  $(GREEN)make build$(NC)              - Build all contracts\n"
	@printf "  $(GREEN)make test$(NC)               - Build and test all contracts\n"
	@printf "  $(GREEN)make fmt$(NC)                - Format all code\n"
	@printf "  $(GREEN)make clean$(NC)              - Clean all build artifacts\n"
	@printf "\n"
	@printf "$(YELLOW)Deployment targets:$(NC)\n"
	@printf "  $(GREEN)make deploy-all$(NC)         - Deploy entire protocol\n"
	@printf "  $(GREEN)make deploy-core$(NC)        - Deploy core contracts only\n"
	@printf "  $(GREEN)make deploy-adapters$(NC)    - Deploy adapter contracts\n"
	@printf "  $(GREEN)make configure-all$(NC)      - Configure entire protocol\n"
	@printf "  $(GREEN)make quick-deploy$(NC)       - Build and deploy everything\n"
	@printf "\n"
	@printf "$(YELLOW)Individual deployment:$(NC)\n"
	@printf "  $(GREEN)make deploy-cusd-manager$(NC)\n"
	@printf "  $(GREEN)make deploy-registry$(NC)\n"
	@printf "  $(GREEN)make deploy-distributor$(NC)\n"
	@printf "  $(GREEN)make deploy-controller$(NC)\n"
	@printf "  $(GREEN)make deploy-blend-adapter$(NC)\n"
	@printf "\n"
	@printf "$(YELLOW)Configuration:$(NC)\n"
	@printf "  $(GREEN)make configure-cusd$(NC)\n"
	@printf "  $(GREEN)make cusd-set-controller$(NC)\n"
	@printf "  $(GREEN)make cusd-set-admin-issuer$(NC)\n"
	@printf "  $(GREEN)make cusd-set-manager-issuer$(NC)\n"
	@printf "  $(GREEN)make configure-distributor$(NC)\n"
	@printf "  $(GREEN)make register-blend-adapter$(NC)\n"
	@printf "\n"
	@printf "$(YELLOW)Utilities:$(NC)\n"
	@printf "  $(GREEN)make show-addresses$(NC)     - Show deployed addresses\n"
	@printf "  $(GREEN)make save-addresses$(NC)     - Save addresses to file\n"
	@printf "  $(GREEN)make load-existing$(NC)      - Load existing v2 addresses\n"
	@printf "  $(GREEN)make test-deposit$(NC)       - Test deposit operation\n"
	@printf "  $(GREEN)make verify-deployment$(NC)  - Verify deployment status\n"

# ========== BUILD TARGETS ==========

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
	@rm -f deployed_addresses.mk deployed_addresses.sh

# Generate contract bindings
# loop through all the contracts and call the contract bindings typescript command
# export CONTRACT_NAME='lending_yield_controller' && \
# stellar contract bindings typescript \
# --wasm ./target/wasm32v1-none/release/${CONTRACT_NAME}.wasm \
# --output-dir ./ts/${CONTRACT_NAME} \
# --overwrite

# ========== DEPLOYMENT TARGETS ==========

# Check if contracts are built
.PHONY: check-build
check-build:
	@if [ ! -f "$(WASM_DIR)/cusd_manager.wasm" ]; then \
		printf "$(RED)Error: Contracts not built. Run 'make build' first.$(NC)\n"; \
		exit 1; \
	fi
	@printf "$(GREEN)✓ Contracts are built$(NC)\n"

# Deploy all contracts and configure
.PHONY: deploy-all
deploy-all: check-build deploy-core deploy-adapters configure-all
	@printf "$(GREEN)Full protocol deployment complete!$(NC)\n"
	@$(MAKE) show-addresses
	@$(MAKE) save-addresses

# Deploy core contracts
.PHONY: deploy-core
deploy-core: deploy-cusd-manager deploy-registry deploy-distributor deploy-controller
	@printf "$(GREEN)Core contracts deployed!$(NC)\n"

# Deploy adapter contracts
.PHONY: deploy-adapters
deploy-adapters: deploy-blend-adapter
	@printf "$(GREEN)Adapter contracts deployed!$(NC)\n"

# Configure all contracts
.PHONY: configure-all
configure-all: configure-cusd configure-distributor register-blend-adapter
	@printf "$(GREEN)Protocol configuration complete!$(NC)\n"

# Quick deployment with build
.PHONY: quick-deploy
quick-deploy:
	@printf "$(YELLOW)Starting quick deployment with build...$(NC)\n"
	@$(MAKE) build
	@$(MAKE) deploy-all
	@printf "$(GREEN)Quick deployment complete!$(NC)\n"

# ========== INDIVIDUAL CONTRACT DEPLOYMENT ==========

.PHONY: deploy-cusd-manager
deploy-cusd-manager: check-build
	@printf "$(YELLOW)Deploying CUSD Manager...$(NC)\n"
	$(eval CUSD_MANAGER_ID := $(shell stellar contract deploy \
		--wasm $(WASM_DIR)/cusd_manager.wasm \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		--cusd_id $(CUSD_ID) \
		--owner $(OWNER) \
		--admin $(ADMIN)))
	@printf "$(GREEN)CUSD Manager deployed: $(CUSD_MANAGER_ID)$(NC)\n"

.PHONY: deploy-registry
deploy-registry: check-build
	@printf "$(YELLOW)Deploying Yield Adapter Registry...$(NC)\n"
	$(eval YIELD_ADAPTER_REGISTRY_ID := $(shell stellar contract deploy \
		--wasm $(WASM_DIR)/yield_adapter_registry.wasm \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		--admin $(ADMIN)))
	@printf "$(GREEN)Yield Adapter Registry deployed: $(YIELD_ADAPTER_REGISTRY_ID)$(NC)\n"

.PHONY: deploy-distributor
deploy-distributor: check-build
	@printf "$(YELLOW)Deploying Yield Distributor...$(NC)\n"
	$(eval YIELD_DISTRIBUTOR_ID := $(shell stellar contract deploy \
		--wasm $(WASM_DIR)/yield_distributor.wasm \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		--treasury $(TREASURY) \
		--treasury_share_bps $(TREASURY_SHARE_BPS) \
		--yield_controller GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF \
		--distribution_period $(DISTRIBUTION_PERIOD) \
		--owner $(OWNER) \
		--admin $(ADMIN)))
	@printf "$(GREEN)Yield Distributor deployed: $(YIELD_DISTRIBUTOR_ID)$(NC)\n"

.PHONY: deploy-controller
deploy-controller: check-build
	@printf "$(YELLOW)Deploying Lending Yield Controller...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ] || [ -z "$(YIELD_ADAPTER_REGISTRY_ID)" ] || [ -z "$(CUSD_MANAGER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set. Deploy core contracts first or load addresses.$(NC)\n"; \
		exit 1; \
	fi
	$(eval LENDING_YIELD_CONTROLLER_ID := $(shell stellar contract deploy \
		--wasm $(WASM_DIR)/lending_yield_controller.wasm \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		--yield_distributor $(YIELD_DISTRIBUTOR_ID) \
		--adapter_registry $(YIELD_ADAPTER_REGISTRY_ID) \
		--cusd_manager $(CUSD_MANAGER_ID) \
		--admin $(ADMIN) \
		--owner $(OWNER)))
	@printf "$(GREEN)Lending Yield Controller deployed: $(LENDING_YIELD_CONTROLLER_ID)$(NC)\n"

.PHONY: deploy-blend-adapter
deploy-blend-adapter: check-build
	@printf "$(YELLOW)Deploying Blend Capital Adapter...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set. Deploy controller first or load addresses.$(NC)\n"; \
		exit 1; \
	fi
	$(eval BLEND_CAPITAL_ADAPTER_ID := $(shell stellar contract deploy \
		--wasm $(WASM_DIR)/blend_capital_adapter.wasm \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		--yield_controller $(LENDING_YIELD_CONTROLLER_ID) \
		--blend_pool_id $(BLEND_POOL_ID) \
		--blend_token_id $(BLEND_TOKEN_ID)))
	@printf "$(GREEN)Blend Capital Adapter deployed: $(BLEND_CAPITAL_ADAPTER_ID)$(NC)\n"

# ========== CONFIGURATION TARGETS ==========

.PHONY: configure-cusd
configure-cusd:
	@printf "$(YELLOW)Configuring CUSD Manager...$(NC)\n"
	@if [ -z "$(CUSD_MANAGER_ID)" ] || [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(CUSD_ID) \
		-- \
		set_admin \
		--new_admin $(CUSD_MANAGER_ID)
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(CUSD_MANAGER_ID) \
		-- \
		set_yield_controller \
		--caller $(ADMIN) \
		--new_controller $(LENDING_YIELD_CONTROLLER_ID)
	@printf "$(GREEN)CUSD Manager configured!$(NC)\n"

.PHONY: cusd-set-controller
cusd-set-controller:
	@printf "$(YELLOW)Setting Yield Controller...$(NC)\n"
	@if [ -z "$(CUSD_MANAGER_ID)" ] || [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(CUSD_MANAGER_ID) \
		-- \
		set_yield_controller \
		--caller $(ADMIN) \
		--new_controller $(LENDING_YIELD_CONTROLLER_ID)
	@printf "$(GREEN)CUSD Manager configured!$(NC)\n"

.PHONY: cusd-set-admin-issuer
cusd-set-admin-issuer:
	@printf "$(YELLOW)Setting Admin as Issuer...$(NC)\n"
	@if [ -z "$(CUSD_MANAGER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source owner \
		--network testnet \
		--id $(CUSD_MANAGER_ID) \
		-- \
		set_cusd_issuer \
		--caller $(OWNER) \
		--new_issuer $(ADMIN)
	@printf "$(GREEN)CUSD Issuer set!$(NC)\n"

.PHONY: cusd-set-manager-issuer
cusd-set-manager-issuer:
	@printf "$(YELLOW)Setting Manager as Issuer...$(NC)\n"
	@if [ -z "$(CUSD_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source admin \
		--network testnet \
		--id $(CUSD_ID) \
		-- \
		set_admin \
		--new_admin $(CUSD_MANAGER_ID)
	@printf "$(GREEN)CUSD Manager set as Issuer!$(NC)\n"

.PHONY: configure-distributor
configure-distributor:
	@printf "$(YELLOW)Configuring Yield Distributor...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ] || [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		set_yield_controller \
		--caller $(ADMIN) \
		--yield_controller $(LENDING_YIELD_CONTROLLER_ID)
	@printf "$(GREEN)Yield Distributor configured!$(NC)\n"

.PHONY: register-blend-adapter
register-blend-adapter:
	@printf "$(YELLOW)Registering Blend Capital Adapter...$(NC)\n"
	@if [ -z "$(YIELD_ADAPTER_REGISTRY_ID)" ] || [ -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_ADAPTER_REGISTRY_ID) \
		-- \
		register_adapter \
		--caller $(ADMIN) \
		--yield_type "LEND" \
		--protocol "BC_LA" \
		--adapter_address $(BLEND_CAPITAL_ADAPTER_ID)
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_ADAPTER_REGISTRY_ID) \
		-- \
		add_support_for_asset \
		--caller $(ADMIN) \
		--yield_type "LEND" \
		--protocol "BC_LA" \
		--asset_address $(USDC_ID)
	@printf "$(GREEN)Blend adapter registered with USDC support!$(NC)\n"

# ========== UTILITY TARGETS ==========

.PHONY: test-deposit
test-deposit:
	@printf "$(YELLOW)Testing deposit operation...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	# stellar contract invoke \
	# 	--source $(ADMIN_KEY) \
	# 	--network $(NETWORK) \
	# 	--id $(USDC_ID) \
	# 	-- \
	# 	approve \
	# 	--from $(ADMIN) \
	# 	--spender $(LENDING_YIELD_CONTROLLER_ID) \
	# 	--amount 10000000 \
	# 	--expiration_ledger $(EXPIRATION_LEDGER)
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		deposit_collateral \
		--protocol "BC_LA" \
		--user $(ADMIN) \
		--asset $(USDC_ID) \
		--amount 10000000
	@printf "$(GREEN)Test deposit complete!$(NC)\n"

.PHONY: test-withdraw
test-withdraw::
	@printf "$(YELLOW)Testing deposit operation...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	# stellar contract invoke \
	# 	--source $(ADMIN_KEY) \
	# 	--network $(NETWORK) \
	# 	--id $(USDC_ID) \
	# 	-- \
	# 	approve \
	# 	--from $(ADMIN) \
	# 	--spender $(LENDING_YIELD_CONTROLLER_ID) \
	# 	--amount 10000000 \
	# 	--expiration_ledger $(EXPIRATION_LEDGER)
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		withdraw_collateral \
		--protocol "BC_LA" \
		--user $(ADMIN) \
		--asset $(USDC_ID) \
		--amount 10000000
	@printf "$(GREEN)Test deposit complete!$(NC)\n"

.PHONY: test-get-yield
test-get-yield::
	@printf "$(YELLOW)Testing deposit operation...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	# stellar contract invoke \
	# 	--source $(ADMIN_KEY) \
	# 	--network $(NETWORK) \
	# 	--id $(USDC_ID) \
	# 	-- \
	# 	approve \
	# 	--from $(ADMIN) \
	# 	--spender $(LENDING_YIELD_CONTROLLER_ID) \
	# 	--amount 10000000 \
	# 	--expiration_ledger $(EXPIRATION_LEDGER)
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		deposit_collateral \
		--protocol "BC_LA" \
		--user $(ADMIN) \
		--asset $(USDC_ID) \
		--amount 10000000
	@printf "$(GREEN)Test deposit complete!$(NC)\n"

.PHONY: show-addresses
show-addresses:
	@printf "$(YELLOW)Contract Addresses:$(NC)\n"
	@printf "$(GREEN)External Contracts:$(NC)\n"
	@printf "  CUSD_ID                      = $(CUSD_ID)\n"
	@printf "  USDC_ID                      = $(USDC_ID)\n"
	@printf "  BLEND_POOL_ID                = $(BLEND_POOL_ID)\n"
	@printf "  BLEND_TOKEN_ID               = $(BLEND_TOKEN_ID)\n"
	@printf "\n"
	@printf "$(GREEN)Deployed Contracts:$(NC)\n"
	@printf "  CUSD_MANAGER_ID              = $(if $(CUSD_MANAGER_ID),$(CUSD_MANAGER_ID),$(RED)Not deployed$(NC))\n"
	@printf "  YIELD_ADAPTER_REGISTRY_ID    = $(if $(YIELD_ADAPTER_REGISTRY_ID),$(YIELD_ADAPTER_REGISTRY_ID),$(RED)Not deployed$(NC))\n"
	@printf "  YIELD_DISTRIBUTOR_ID         = $(if $(YIELD_DISTRIBUTOR_ID),$(YIELD_DISTRIBUTOR_ID),$(RED)Not deployed$(NC))\n"
	@printf "  LENDING_YIELD_CONTROLLER_ID  = $(if $(LENDING_YIELD_CONTROLLER_ID),$(LENDING_YIELD_CONTROLLER_ID),$(RED)Not deployed$(NC))\n"
	@printf "  BLEND_CAPITAL_ADAPTER_ID     = $(if $(BLEND_CAPITAL_ADAPTER_ID),$(BLEND_CAPITAL_ADAPTER_ID),$(RED)Not deployed$(NC))\n"

.PHONY: save-addresses
save-addresses:
	@printf "$(YELLOW)Saving addresses...$(NC)\n"
	@echo "# Deployed contract addresses - $(shell date)" > deployed_addresses.mk
	@if [ ! -z "$(CUSD_MANAGER_ID)" ]; then echo "CUSD_MANAGER_ID = $(CUSD_MANAGER_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(YIELD_ADAPTER_REGISTRY_ID)" ]; then echo "YIELD_ADAPTER_REGISTRY_ID = $(YIELD_ADAPTER_REGISTRY_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(YIELD_DISTRIBUTOR_ID)" ]; then echo "YIELD_DISTRIBUTOR_ID = $(YIELD_DISTRIBUTOR_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then echo "LENDING_YIELD_CONTROLLER_ID = $(LENDING_YIELD_CONTROLLER_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then echo "BLEND_CAPITAL_ADAPTER_ID = $(BLEND_CAPITAL_ADAPTER_ID)" >> deployed_addresses.mk; fi
	@echo "#!/bin/bash" > deployed_addresses.sh
	@echo "# Deployed contract addresses - $(shell date)" >> deployed_addresses.sh
	@if [ ! -z "$(CUSD_MANAGER_ID)" ]; then echo "export CUSD_MANAGER_ID=$(CUSD_MANAGER_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(YIELD_ADAPTER_REGISTRY_ID)" ]; then echo "export YIELD_ADAPTER_REGISTRY_ID=$(YIELD_ADAPTER_REGISTRY_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(YIELD_DISTRIBUTOR_ID)" ]; then echo "export YIELD_DISTRIBUTOR_ID=$(YIELD_DISTRIBUTOR_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then echo "export LENDING_YIELD_CONTROLLER_ID=$(LENDING_YIELD_CONTROLLER_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then echo "export BLEND_CAPITAL_ADAPTER_ID=$(BLEND_CAPITAL_ADAPTER_ID)" >> deployed_addresses.sh; fi
	@chmod +x deployed_addresses.sh
	@printf "$(GREEN)Addresses saved to deployed_addresses.mk and deployed_addresses.sh$(NC)\n"

.PHONY: load-existing
load-existing:
	@printf "$(YELLOW)Loading existing v2 addresses from testnet.json...$(NC)\n"
	@echo "# Loaded from testnet.json v2 - $(shell date)" > deployed_addresses.mk
	@echo "CUSD_MANAGER_ID = CBPQVQ6KFEQMB4W25AJT6MWPINFRJO4CYCJEEA6H5M7UUT5TLXISUCAY" >> deployed_addresses.mk
	@echo "YIELD_ADAPTER_REGISTRY_ID = CCMBIA6M2FLCKE4USF2TAU5TAED23TPPPW7JBATGBMF5JI4L5ML4EKHL" >> deployed_addresses.mk
	@echo "YIELD_DISTRIBUTOR_ID = CBHJBD7PSM524MBLIRBKMVNBDK4EFPS7T4XIPA5OCC4E2BRPM4ZXYUVG" >> deployed_addresses.mk
	@echo "LENDING_YIELD_CONTROLLER_ID = CDTZYUNULCB426ONSR3XRK75RHKUBJDQWMRRPC4POPJH3PWD46KBDF2M" >> deployed_addresses.mk
	@echo "BLEND_CAPITAL_ADAPTER_ID = CC2JU4VDRYDEOMW62PV4GM3EXD4ODVIUHD6LSO6DUD5XEH5TQOUKSQCA" >> deployed_addresses.mk
	@printf "$(GREEN)Existing addresses loaded!$(NC)\n"

.PHONY: verify-deployment
verify-deployment:
	@printf "$(YELLOW)Verifying deployment...$(NC)\n"
	@printf "\n"
	@printf "$(YELLOW)Build Status:$(NC)\n"
	@if [ -f "$(WASM_DIR)/cusd_manager.wasm" ]; then \
		printf "$(GREEN)✓ Contracts built$(NC)\n"; \
	else \
		printf "$(RED)✗ Contracts not built - run 'make build'$(NC)\n"; \
	fi
	@printf "\n"
	@printf "$(YELLOW)Deployment Status:$(NC)\n"
	@if [ ! -z "$(CUSD_MANAGER_ID)" ]; then \
		printf "$(GREEN)✓ CUSD Manager: $(CUSD_MANAGER_ID)$(NC)\n"; \
	else \
		printf "$(RED)✗ CUSD Manager not deployed$(NC)\n"; \
	fi
	@if [ ! -z "$(YIELD_ADAPTER_REGISTRY_ID)" ]; then \
		printf "$(GREEN)✓ Yield Adapter Registry: $(YIELD_ADAPTER_REGISTRY_ID)$(NC)\n"; \
	else \
		printf "$(RED)✗ Yield Adapter Registry not deployed$(NC)\n"; \
	fi
	@if [ ! -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(GREEN)✓ Yield Distributor: $(YIELD_DISTRIBUTOR_ID)$(NC)\n"; \
	else \
		printf "$(RED)✗ Yield Distributor not deployed$(NC)\n"; \
	fi
	@if [ ! -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(GREEN)✓ Lending Yield Controller: $(LENDING_YIELD_CONTROLLER_ID)$(NC)\n"; \
	else \
		printf "$(RED)✗ Lending Yield Controller not deployed$(NC)\n"; \
	fi
	@if [ ! -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then \
		printf "$(GREEN)✓ Blend Capital Adapter: $(BLEND_CAPITAL_ADAPTER_ID)$(NC)\n"; \
	else \
		printf "$(RED)✗ Blend Capital Adapter not deployed$(NC)\n"; \
	fi

# ========== REDEPLOY TARGETS ==========

.PHONY: redeploy-cusd-manager
redeploy-cusd-manager: deploy-cusd-manager configure-cusd save-addresses
	@printf "$(GREEN)CUSD Manager redeployed and configured!$(NC)\n"

.PHONY: redeploy-registry
redeploy-registry: deploy-registry register-blend-adapter save-addresses
	@printf "$(GREEN)Registry redeployed and configured!$(NC)\n"

.PHONY: redeploy-distributor
redeploy-distributor: deploy-distributor configure-distributor save-addresses
	@printf "$(GREEN)Distributor redeployed and configured!$(NC)\n"

.PHONY: redeploy-controller
redeploy-controller: deploy-controller configure-cusd configure-distributor save-addresses
	@printf "$(GREEN)Controller redeployed and configured!$(NC)\n"

.PHONY: redeploy-blend-adapter
redeploy-blend-adapter: deploy-blend-adapter register-blend-adapter save-addresses
	@printf "$(GREEN)Blend adapter redeployed and registered!$(NC)\n"

.DEFAULT_GOAL := help