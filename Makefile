# Coopstable Protocol Makefile - Build and Deploy
# This Makefile handles both building and deployment of the Coopstable protocol

# Build configuration
ALL_DIRS := packages/blend_capital_adapter packages/yield_adapter contracts/cusd_manager contracts/yield_adapter_registry contracts/yield_distributor contracts/lending_yield_controller
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
CUSD_CODE ?= CUSD

# Get public keys from stellar keys
OWNER := $(shell stellar keys public-key $(OWNER_KEY))
ADMIN := $(shell stellar keys public-key $(ADMIN_KEY))
TREASURY := $(shell stellar keys public-key $(TREASURY_KEY))

# External Contract IDs (pre-deployed on testnet)
BLEND_POOL_ID = CCLBPEYS3XFK65MYYXSBMOGKUI4ODN5S7SUZBGD7NALUQF64QILLX5B5
BLEND_TOKEN_ID = CB22KRA3YZVCNCQI64JQ5WE7UY2VAV7WFLK6A2JN3HEX56T2EDAFO7QF
USDC_ID = CAQCFVLOBK5GIULPNZRGATJJMIZL5BSP7X5YJVMGCPTUEPFM4AVSRCJU
CUSD_ADDRESS = $(CUSD_CODE):$(OWNER)

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
	@printf "$(YELLOW)Full Protocol Deployment:$(NC)\n"
	@printf "  $(GREEN)make deploy-protocol$(NC)    - Deploy entire protocol with dependencies\n"
	@printf "  $(GREEN)make quick-deploy$(NC)       - Build and deploy everything\n"
	@printf "  $(GREEN)make redeploy-protocol$(NC)  - Redeploy and reconfigure entire protocol\n"
	@printf "\n"
	@printf "$(YELLOW)Individual Contract Deployment:$(NC)\n"
	@printf "  $(GREEN)make deploy-cusd-full$(NC)      - Deploy CUSD Token with deps\n"
	@printf "  $(GREEN)make deploy-cusd-manager-full$(NC)     - Deploy CUSD Manager with deps\n"
	@printf "  $(GREEN)make deploy-registry-full$(NC)        - Deploy Registry with deps\n"
	@printf "  $(GREEN)make deploy-distributor-full$(NC)     - Deploy Distributor with deps\n"
	@printf "  $(GREEN)make deploy-controller-full$(NC)      - Deploy Controller with deps\n"
	@printf "  $(GREEN)make deploy-blend-adapter-full$(NC)   - Deploy Blend Adapter with deps\n"
	@printf "\n"
	@printf "$(YELLOW)Basic Deployment (no dependencies):$(NC)\n"
	@printf "  $(GREEN)make deploy-all$(NC)         - Deploy core + adapters + configure\n"
	@printf "  $(GREEN)make deploy-core$(NC)        - Deploy core contracts only\n"
	@printf "  $(GREEN)make deploy-adapters$(NC)    - Deploy adapter contracts\n"
	@printf "  $(GREEN)make configure-all$(NC)      - Configure entire protocol\n"
	@printf "\n"
	@printf "$(YELLOW)Protocol Testing:$(NC)\n"
	@printf "  $(GREEN)make test-read-yield$(NC)    - Read current yield from protocols\n"
	@printf "  $(GREEN)make test-deposit$(NC)       - Test collateral deposit operation\n"
	@printf "  $(GREEN)make test-withdraw$(NC)      - Test collateral withdrawal\n"
	@printf "  $(GREEN)make test-claim-yield$(NC)   - Test yield claiming and distribution\n"
	@printf "  $(GREEN)make test-full-cycle$(NC)    - Test complete deposit->yield->withdraw cycle\n"
	@printf "\n"
	@printf "$(YELLOW)Configuration:$(NC)\n"
	@printf "  $(GREEN)make configure-cusd$(NC)\n"
	@printf "  $(GREEN)make configure-distributor$(NC)\n"
	@printf "  $(GREEN)make register-blend-adapter$(NC)\n"
	@printf "\n"
	@printf "$(YELLOW)Utilities:$(NC)\n"
	@printf "  $(GREEN)make show-addresses$(NC)     - Show deployed addresses\n"
	@printf "  $(GREEN)make save-addresses$(NC)     - Save addresses to file\n"
	@printf "  $(GREEN)make verify-deployment$(NC)  - Verify deployment status\n"
	@printf "  $(GREEN)make get-balances$(NC)       - Show token balances\n"

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

# ========== FULL PROTOCOL DEPLOYMENT ==========

# Deploy entire protocol with proper dependency order
.PHONY: deploy-protocol
deploy-protocol: check-build
	@printf "$(YELLOW)Starting full protocol deployment with dependency management...$(NC)\n"
	@$(MAKE) deploy-cusd-full
	@$(MAKE) deploy-cusd-manager-full
	@$(MAKE) deploy-registry-full
	@$(MAKE) deploy-distributor-full
	@$(MAKE) deploy-controller-full
	@$(MAKE) deploy-blend-adapter-full
	@printf "$(GREEN)Full protocol deployment complete!$(NC)\n"
	@$(MAKE) show-addresses
	@$(MAKE) save-addresses

# Redeploy entire protocol (clean deployment)
.PHONY: redeploy-protocol
redeploy-protocol:
	@printf "$(YELLOW)Starting protocol redeployment...$(NC)\n"
	@# Transfer CUSD issuer back to owner before clean if CUSD Manager exists
	@if [ -f deployed_addresses.mk ] && grep -q "CUSD_MANAGER_ID" deployed_addresses.mk; then \
		CUSD_MANAGER_ID=$$(grep '^CUSD_MANAGER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
		if [ ! -z "$$CUSD_MANAGER_ID" ]; then \
			printf "$(YELLOW)Transferring CUSD issuer role back to owner...$(NC)\n"; \
			stellar contract invoke \
				--source $(ADMIN_KEY) \
				--network $(NETWORK) \
				--id $$CUSD_MANAGER_ID \
				-- \
				set_cusd_issuer \
				--new_issuer $(OWNER) || printf "$(RED)Warning: Failed to transfer issuer role$(NC)\n"; \
		fi \
	fi
	@$(MAKE) clean
	@$(MAKE) build
	@$(MAKE) deploy-protocol
	@printf "$(GREEN)Protocol redeployment complete!$(NC)\n"

# Quick deployment with build
.PHONY: quick-deploy
quick-deploy:
	@printf "$(YELLOW)Starting quick deployment with build...$(NC)\n"
	@$(MAKE) build
	@$(MAKE) deploy-protocol
	@printf "$(GREEN)Quick deployment complete!$(NC)\n"

# ========== INDIVIDUAL CONTRACT DEPLOYMENT WITH DEPENDENCIES ==========

# Deploy CUSD Token with all dependencies and setup
.PHONY: deploy-cusd-full
deploy-cusd-full: check-build
	@printf "$(YELLOW)Deploying CUSD Asset with full setup...$(NC)\n"
	@$(MAKE) deploy-cusd
	@printf "$(GREEN)CUSD Asset deployed and ready!$(NC)\n"

# Deploy CUSD Manager with all dependencies and setup
.PHONY: deploy-cusd-manager-full
deploy-cusd-manager-full: check-build
	@printf "$(YELLOW)Deploying CUSD Manager with full setup...$(NC)\n"
	@$(MAKE) deploy-cusd-manager
	@printf "$(YELLOW)Configuring CUSD Asset with new manager...$(NC)\n"
	@$(MAKE) configure-cusd
	@printf "$(GREEN)CUSD Manager deployed and configured!$(NC)\n"

# Deploy Registry with all dependencies and setup
.PHONY: deploy-registry-full
deploy-registry-full: check-build
	@printf "$(YELLOW)Deploying Registry with full setup...$(NC)\n"
	@$(MAKE) deploy-registry
	@printf "$(GREEN)Registry deployed and ready!$(NC)\n"

# Deploy Distributor with all dependencies and setup
.PHONY: deploy-distributor-full
deploy-distributor-full: check-build
	@printf "$(YELLOW)Deploying Distributor with full setup...$(NC)\n"
	@$(MAKE) deploy-distributor
	@printf "$(GREEN)Distributor deployed and ready!$(NC)\n"

# Deploy Controller with all dependencies and setup
.PHONY: deploy-controller-full
deploy-controller-full: check-build
	@printf "$(YELLOW)Deploying Controller with full setup...$(NC)\n"
	@DISTRIBUTOR_ID=$$(grep '^YIELD_DISTRIBUTOR_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	REGISTRY_ID=$$(grep '^YIELD_ADAPTER_REGISTRY_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	MANAGER_ID=$$(grep '^CUSD_MANAGER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$DISTRIBUTOR_ID" ] || [ -z "$$REGISTRY_ID" ] || [ -z "$$MANAGER_ID" ]; then \
		printf "$(RED)Error: Dependencies not deployed. Run deploy-cusd-manager-full, deploy-registry-full, and deploy-distributor-full first.$(NC)\n"; \
		exit 1; \
	fi; \
	$(MAKE) -e YIELD_DISTRIBUTOR_ID="$$DISTRIBUTOR_ID" YIELD_ADAPTER_REGISTRY_ID="$$REGISTRY_ID" CUSD_MANAGER_ID="$$MANAGER_ID" deploy-controller
	@$(MAKE) configure-cusd
	@$(MAKE) cusd-manager-set-controller
	@$(MAKE) configure-distributor
	@printf "$(GREEN)Controller deployed and configured!$(NC)\n"

# Deploy Blend Adapter with all dependencies and setup
.PHONY: deploy-blend-adapter-full
deploy-blend-adapter-full: check-build
	@printf "$(YELLOW)Deploying Blend Adapter with full setup...$(NC)\n"
	@CONTROLLER_ID=$$(grep '^LENDING_YIELD_CONTROLLER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$CONTROLLER_ID" ]; then \
		printf "$(RED)Error: Controller not deployed. Run deploy-controller-full first.$(NC)\n"; \
		exit 1; \
	fi; \
	$(MAKE) -e LENDING_YIELD_CONTROLLER_ID="$$CONTROLLER_ID" deploy-blend-adapter
	@$(MAKE) register-blend-adapter
	@printf "$(GREEN)Blend Adapter deployed and registered!$(NC)\n"

# ========== BASIC DEPLOYMENT TARGETS ==========

# Deploy all contracts and configure
.PHONY: deploy-all
deploy-all: check-build deploy-core deploy-adapters configure-all
	@printf "$(GREEN)Basic deployment complete!$(NC)\n"
	@$(MAKE) show-addresses
	@$(MAKE) save-addresses

# Deploy core contracts
.PHONY: deploy-core
deploy-core: deploy-cusd deploy-cusd-manager deploy-registry deploy-distributor deploy-controller
	@printf "$(GREEN)Core contracts deployed!$(NC)\n"

# Deploy adapter contracts
.PHONY: deploy-adapters
deploy-adapters: deploy-blend-adapter
	@printf "$(GREEN)Adapter contracts deployed!$(NC)\n"

# Configure all contracts
.PHONY: configure-all
configure-all: configure-cusd configure-distributor register-blend-adapter
	@printf "$(GREEN)Protocol configuration complete!$(NC)\n"

# ========== INDIVIDUAL CONTRACT DEPLOYMENT ==========

.PHONY: deploy-cusd
deploy-cusd: check-build
	@printf "$(YELLOW)Deploying CUSD Token...$(NC)\n"
	# creates the stellar asset ensures it exists
	stellar tx new payment \
		--source-account owner \
		--destination $$(stellar keys public-key owner) \
		--asset CUSD:$$(stellar keys public-key owner) \
		--amount 1000 \
		--fee 1000 \
		--network $(NETWORK)
	@CUSD_ID=$$(stellar contract asset deploy \
		--source-account owner \
		--alias cusd_token \
		--fee 1000 \
		--network testnet \
		--asset CUSD:$$(stellar keys public-key owner) 2>/dev/null || \
		stellar contract id asset --asset CUSD:$$(stellar keys public-key owner)); \
	for account in admin treasury member_1 member_2 member_3; do \
		echo "Setting trustline for $$account..."; \
		stellar tx new change-trust \
			--source-account $$account \
			--line CUSD:$$(stellar keys public-key owner) \
			--fee 1000 \
			--network testnet; \
	done; \
	printf "$(GREEN)CUSD Token deployed: $$CUSD_ID$(NC)\n"; \
	echo "# Deployed contract addresses - $$(date)" > deployed_addresses.mk.tmp; \
	echo "CUSD_ID = $$CUSD_ID" >> deployed_addresses.mk.tmp; \
	if [ -f deployed_addresses.mk ]; then grep -v "^CUSD_ID" deployed_addresses.mk | grep -v "^#" >> deployed_addresses.mk.tmp || true; fi; \
	mv deployed_addresses.mk.tmp deployed_addresses.mk

.PHONY: deploy-cusd-manager
deploy-cusd-manager: check-build
	@printf "$(YELLOW)Deploying CUSD Manager...$(NC)\n"
	@if [ -f deployed_addresses.mk ]; then \
		CUSD_ID=$$(grep '^CUSD_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
		if [ -z "$$CUSD_ID" ]; then \
			printf "$(RED)Error: CUSD_ID not found in deployed_addresses.mk. Please deploy CUSD first.$(NC)\n"; \
			exit 1; \
		fi; \
		CUSD_MANAGER_ID=$$(stellar contract deploy \
			--wasm $(WASM_DIR)/cusd_manager.wasm \
			--source $(OWNER_KEY) \
			--network $(NETWORK) \
			-- \
			--cusd_id $$CUSD_ID \
			--owner $(OWNER) \
			--admin $(ADMIN)); \
		printf "$(GREEN)CUSD Manager deployed: $$CUSD_MANAGER_ID$(NC)\n"; \
		echo "# Deployed contract addresses - $$(date)" > deployed_addresses.mk.tmp; \
		echo "CUSD_MANAGER_ID = $$CUSD_MANAGER_ID" >> deployed_addresses.mk.tmp; \
		grep -v "^CUSD_MANAGER_ID" deployed_addresses.mk | grep -v "^#" >> deployed_addresses.mk.tmp || true; \
		mv deployed_addresses.mk.tmp deployed_addresses.mk; \
	else \
		printf "$(RED)Error: deployed_addresses.mk not found. Please deploy CUSD first.$(NC)\n"; \
		exit 1; \
	fi

.PHONY: deploy-registry
deploy-registry: check-build
	@printf "$(YELLOW)Deploying Yield Adapter Registry...$(NC)\n"
	$(eval YIELD_ADAPTER_REGISTRY_ID := $(shell stellar contract deploy \
		--wasm $(WASM_DIR)/yield_adapter_registry.wasm \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		--admin $(ADMIN) \
		--owner $(OWNER)))
	@printf "$(GREEN)Yield Adapter Registry deployed: $(YIELD_ADAPTER_REGISTRY_ID)$(NC)\n"
	@echo "# Deployed contract addresses - $$(date)" > deployed_addresses.mk.tmp
	@echo "YIELD_ADAPTER_REGISTRY_ID = $(YIELD_ADAPTER_REGISTRY_ID)" >> deployed_addresses.mk.tmp
	@if [ -f deployed_addresses.mk ]; then grep -v "^YIELD_ADAPTER_REGISTRY_ID" deployed_addresses.mk | grep -v "^#" >> deployed_addresses.mk.tmp || true; fi
	@mv deployed_addresses.mk.tmp deployed_addresses.mk

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
	@echo "# Deployed contract addresses - $$(date)" > deployed_addresses.mk.tmp
	@echo "YIELD_DISTRIBUTOR_ID = $(YIELD_DISTRIBUTOR_ID)" >> deployed_addresses.mk.tmp
	@if [ -f deployed_addresses.mk ]; then grep -v "^YIELD_DISTRIBUTOR_ID" deployed_addresses.mk | grep -v "^#" >> deployed_addresses.mk.tmp || true; fi
	@mv deployed_addresses.mk.tmp deployed_addresses.mk

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
	@echo "# Deployed contract addresses - $$(date)" > deployed_addresses.mk.tmp
	@echo "LENDING_YIELD_CONTROLLER_ID = $(LENDING_YIELD_CONTROLLER_ID)" >> deployed_addresses.mk.tmp
	@if [ -f deployed_addresses.mk ]; then grep -v "^LENDING_YIELD_CONTROLLER_ID" deployed_addresses.mk | grep -v "^#" >> deployed_addresses.mk.tmp || true; fi
	@mv deployed_addresses.mk.tmp deployed_addresses.mk

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
	@echo "# Deployed contract addresses - $$(date)" > deployed_addresses.mk.tmp
	@echo "BLEND_CAPITAL_ADAPTER_ID = $(BLEND_CAPITAL_ADAPTER_ID)" >> deployed_addresses.mk.tmp
	@if [ -f deployed_addresses.mk ]; then grep -v "^BLEND_CAPITAL_ADAPTER_ID" deployed_addresses.mk | grep -v "^#" >> deployed_addresses.mk.tmp || true; fi
	@mv deployed_addresses.mk.tmp deployed_addresses.mk

# ========== CONFIGURATION TARGETS ==========
.PHONY: configure-cusd
configure-cusd:
	@printf "$(YELLOW)Configuring CUSD Asset...$(NC)\n"
	@CUSD_ID=$$(grep '^CUSD_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	CUSD_MANAGER_ID=$$(grep '^CUSD_MANAGER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$CUSD_ID" ] || [ -z "$$CUSD_MANAGER_ID" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi; \
	printf "$(YELLOW)Setting CUSD Manager ($$CUSD_MANAGER_ID) as CUSD Issuer...$(NC)\n"; \
	stellar contract invoke \
		--source $(OWNER_KEY) \
		--network $(NETWORK) \
		--id $$CUSD_ID \
		-- \
		set_admin \
		--new_admin $$CUSD_MANAGER_ID; \
	printf "$(YELLOW)Checking Owner CUSD balance...$(NC)\n"; \
	OWNER_BALANCE=$$(stellar contract invoke \
		--id $$CUSD_ID \
		--source-account $(OWNER_KEY) \
		--network $(NETWORK) \
		-- \
		balance \
		--id $$(stellar keys public-key $(OWNER_KEY)) 2>/dev/null || echo "0"); \
	if [ "$$OWNER_BALANCE" != "0" ] && [ "$$OWNER_BALANCE" != "" ]; then \
		printf "$(YELLOW)Transferring Owner CUSD balance ($$OWNER_BALANCE) to Admin...$(NC)\n"; \
		stellar contract invoke \
			--id $$CUSD_ID \
			--source-account $(OWNER_KEY) \
			--network $(NETWORK) \
			-- \
			transfer \
			--from $$(stellar keys public-key $(OWNER_KEY)) \
			--to $(ADMIN) \
			--amount $$OWNER_BALANCE; \
	fi
	@printf "$(GREEN)CUSD configured!$(NC)\n"

.PHONY: cusd-manager-set-controller
cusd-manager-set-controller:
	@printf "$(YELLOW)Setting Yield Controller...$(NC)\n"
	@CUSD_MANAGER_ID=$$(grep '^CUSD_MANAGER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	LENDING_YIELD_CONTROLLER_ID=$$(grep '^LENDING_YIELD_CONTROLLER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$CUSD_MANAGER_ID" ] || [ -z "$$LENDING_YIELD_CONTROLLER_ID" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi; \
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $$CUSD_MANAGER_ID \
		-- \
		set_yield_controller \
		--new_controller $$LENDING_YIELD_CONTROLLER_ID
	@printf "$(GREEN)CUSD Manager configured with yield controller!$(NC)\n"

.PHONY: cusd-manager-set-issuer
cusd-manager-set-issuer:
	@printf "$(YELLOW)Setting Admin as Issuer...$(NC)\n"
	@if [ -z "$(CUSD_MANAGER_ID)" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(CUSD_MANAGER_ID) \
		-- \
		set_cusd_issuer \
		--new_issuer $(CUSD_MANAGER_ID)
	@printf "$(GREEN)CUSD Issuer set!$(NC)\n"

.PHONY: cusd-manager-set-admin
cusd-manager-set-admin:
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

.PHONY: configure-cusd-manager
configure-cusd-manager:
	@printf "$(YELLOW)Configuring CUSD Manager...$(NC)\n"
	@CUSD_MANAGER_ID=$$(grep '^CUSD_MANAGER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$CUSD_MANAGER_ID" ]; then \
		printf "$(RED)Error: Required contract ID not set.$(NC)\n"; \
		exit 1; \
	fi
	@$(MAKE) configure-cusd
	@$(MAKE) cusd-manager-set-issuer

.PHONY: configure-distributor
configure-distributor:
	@printf "$(YELLOW)Configuring Yield Distributor...$(NC)\n"
	@DISTRIBUTOR_ID=$$(grep '^YIELD_DISTRIBUTOR_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	CONTROLLER_ID=$$(grep '^LENDING_YIELD_CONTROLLER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$DISTRIBUTOR_ID" ] || [ -z "$$CONTROLLER_ID" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi; \
	printf "$(YELLOW)Setting Yield Controller ($$CONTROLLER_ID) in Yield Distributor ($$DISTRIBUTOR_ID)...$(NC)\n"; \
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $$DISTRIBUTOR_ID \
		-- \
		set_yield_controller \
		--yield_controller $$CONTROLLER_ID
	@printf "$(GREEN)Yield Distributor configured!$(NC)\n"


.PHONY: register-blend-adapter
register-blend-adapter:
	@printf "$(YELLOW)Registering Blend Capital Adapter...$(NC)\n"
	@REGISTRY_ID=$$(grep '^YIELD_ADAPTER_REGISTRY_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	ADAPTER_ID=$$(grep '^BLEND_CAPITAL_ADAPTER_ID' deployed_addresses.mk | cut -d'=' -f2 | tr -d ' '); \
	if [ -z "$$REGISTRY_ID" ] || [ -z "$$ADAPTER_ID" ]; then \
		printf "$(RED)Error: Required contract IDs not set.$(NC)\n"; \
		exit 1; \
	fi; \
	printf "$(YELLOW)Registering Blend Capital Adapter ($$ADAPTER_ID) in Registry ($$REGISTRY_ID)...$(NC)\n"; \
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $$REGISTRY_ID \
		-- \
		register_adapter \
		--yield_type "LEND" \
		--protocol "BC_LA" \
		--adapter_address $$ADAPTER_ID; \
	printf "$(YELLOW)Adding USDC asset support...$(NC)\n"; \
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $$REGISTRY_ID \
		-- \
		add_support_for_asset \
		--yield_type "LEND" \
		--protocol "BC_LA" \
		--asset_address $(USDC_ID)
	@printf "$(GREEN)Blend adapter registered with USDC support!$(NC)\n"

# ========== PROTOCOL TESTING TARGETS ==========

# Test amount for operations (1 USDC = 10000000 stroops)
TEST_AMOUNT ?= 9000000000

# Read current yield from all protocols
.PHONY: test-read-yield
test-read-yield:
	@printf "$(YELLOW)Reading current yield from protocols...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	@printf "$(YELLOW)Getting total yield across all protocols:$(NC)\n"
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		get_yield
	@printf "$(YELLOW)Getting yield from Blend Capital adapter directly:$(NC)\n"
	@if [ ! -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then \
		stellar contract invoke \
			--source $(ADMIN_KEY) \
			--network $(NETWORK) \
			--id $(BLEND_CAPITAL_ADAPTER_ID) \
			-- \
			get_yield \
			--asset $(USDC_ID); \
	else \
		printf "$(RED)Blend Capital Adapter not deployed$(NC)\n"; \
	fi
	@printf "$(GREEN)Yield reading complete!$(NC)\n"

# Test collateral deposit operation
.PHONY: test-deposit
test-deposit:
	@printf "$(YELLOW)Testing collateral deposit ($(TEST_AMOUNT) USDC)...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	@printf "$(YELLOW)Depositing $(TEST_AMOUNT) USDC to Blend Capital via protocol:$(NC)\n"
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		deposit_collateral \
		--protocol "BC_LA" \
		--user $(ADMIN) \
		--asset $(USDC_ID) \
		--amount $(TEST_AMOUNT)
	@printf "$(GREEN)Deposit test complete! User should have received cUSD tokens.$(NC)\n"

# Test collateral withdrawal operation  
.PHONY: test-withdraw
test-withdraw:
	@printf "$(YELLOW)Testing collateral withdrawal ($(TEST_AMOUNT) USDC)...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	@printf "$(YELLOW)Withdrawing $(TEST_AMOUNT) USDC from Blend Capital via protocol:$(NC)\n"
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		withdraw_collateral \
		--protocol "BC_LA" \
		--user $(ADMIN) \
		--asset $(USDC_ID) \
		--amount $(TEST_AMOUNT)
	@printf "$(GREEN)Withdrawal test complete! User should have received USDC and burned cUSD.$(NC)\n"

# Test collateral withdrawal operation  
.PHONY: withdraw
withdraw:
	@printf "$(YELLOW)Testing collateral withdrawal ($(TEST_AMOUNT) USDC)...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	@printf "$(YELLOW)Withdrawing $(TEST_AMOUNT) USDC from Blend Capital via protocol:$(NC)\n"
	stellar contract invoke \
		--source $(ACCOUNT_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		withdraw_collateral \
		--protocol "BC_LA" \
		--user $(ACCOUNT) \
		--asset $(USDC_ID) \
		--amount $(AMOUNT)
	@printf "$(GREEN)Withdrawal test complete! User should have received USDC and burned cUSD.$(NC)\n"

# Test yield claiming and distribution
.PHONY: test-claim-yield
test-claim-yield:
	@printf "$(YELLOW)Testing yield claiming and distribution...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	@printf "$(YELLOW)Step 1: Reading current yield:$(NC)\n"
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		get_yield
	@printf "$(YELLOW)Step 2: Claiming yield and triggering distribution:$(NC)\n"
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		claim_yield
	@printf "$(YELLOW)Step 3: Checking if distribution occurred:$(NC)\n"
	@if [ ! -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		stellar contract invoke \
			--source $(ADMIN_KEY) \
			--network $(NETWORK) \
			--id $(YIELD_DISTRIBUTOR_ID) \
			-- \
			is_distribution_available; \
	fi
	@printf "$(GREEN)Yield claiming test complete!$(NC)\n"

# Test yield claiming and distribution
.PHONY: test-claim-emissions
test-claim-emissions:
	@printf "$(YELLOW)Testing emissions claiming and distribution...$(NC)\n"
	@if [ -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then \
		printf "$(RED)Error: Lending Yield Controller ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(LENDING_YIELD_CONTROLLER_ID) \
		-- \
		claim_emissions \
		--protocol "BC_LA" \
		--asset $(USDC_ID) 
	@printf "$(GREEN)Emissions claiming test complete!$(NC)\n"

.PHONY: test-read-emissions
test-read-emissions:
	@printf "$(YELLOW)Testing emissions querying...$(NC)\n"
	@if [ -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then \
		printf "$(RED)Error: Blend Capital Adapter ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(BLEND_CAPITAL_ADAPTER_ID) \
		-- \
		get_emissions \
		--asset $(USDC_ID) 
	@printf "$(GREEN)Emissions claiming test complete!$(NC)\n"

# Test complete cycle: deposit -> wait -> claim yield -> withdraw
.PHONY: test-full-cycle
test-full-cycle:
	@printf "$(YELLOW)Testing complete deposit -> yield -> withdraw cycle...$(NC)\n"
	@printf "$(YELLOW)Step 1: Get initial balances$(NC)\n"
	@$(MAKE) get-balances
	@printf "$(YELLOW)Step 2: Deposit collateral$(NC)\n"
	@$(MAKE) test-deposit
	@printf "$(YELLOW)Step 3: Check balances after deposit$(NC)\n"
	@$(MAKE) get-balances
	@printf "$(YELLOW)Step 4: Read current yield$(NC)\n"
	@$(MAKE) test-read-yield
	@printf "$(YELLOW)Step 5: Claim yield (if available)$(NC)\n"
	@$(MAKE) test-claim-yield
	@printf "$(YELLOW)Step 6: Check balances after yield claim$(NC)\n"
	@$(MAKE) get-balances
	@printf "$(YELLOW)Step 7: Withdraw collateral$(NC)\n"
	@$(MAKE) test-withdraw
	@printf "$(YELLOW)Step 8: Final balances$(NC)\n"
	@$(MAKE) get-balances
	@printf "$(GREEN)Full cycle test complete!$(NC)\n"

# ========== UTILITY TARGETS ==========

# Get token balances for admin account
.PHONY: get-balances
get-balances:
	@printf "$(YELLOW)Token Balances for $(ADMIN):$(NC)\n"
	@printf "$(GREEN)USDC Balance:$(NC)\n"
	@stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(USDC_ID) \
		-- \
		balance \
		--id $(ADMIN) || printf "$(RED)Error reading USDC balance$(NC)\n"
	@printf "$(GREEN)cUSD Balance:$(NC)\n"
	@stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(CUSD_ID) \
		-- \
		balance \
		--id $(ADMIN) || printf "$(RED)Error reading cUSD balance$(NC)\n"
	@if [ ! -z "$(TREASURY)" ]; then \
		printf "$(YELLOW)Treasury Balances for $(TREASURY):$(NC)\n"; \
		printf "$(GREEN)Treasury USDC Balance:$(NC)\n"; \
		stellar contract invoke \
			--source $(ADMIN_KEY) \
			--network $(NETWORK) \
			--id $(USDC_ID) \
			-- \
			balance \
			--id $(TREASURY) || printf "$(RED)Error reading Treasury USDC balance$(NC)\n"; \
		printf "$(GREEN)Treasury cUSD Balance:$(NC)\n"; \
		stellar contract invoke \
			--source $(ADMIN_KEY) \
			--network $(NETWORK) \
			--id $(CUSD_ID) \
			-- \
			balance \
			--id $(TREASURY) || printf "$(RED)Error reading Treasury cUSD balance$(NC)\n"; \
	fi

.PHONY: show-addresses
show-addresses:
	@printf "$(YELLOW)CoopStable Protocol Contract Addresses:$(NC)\n"
	@printf "\n"
	@printf "$(GREEN)External Contracts (Pre-deployed):$(NC)\n"
	@printf "  USDC Token                   = $(USDC_ID)\n"
	@printf "  Blend Pool                   = $(BLEND_POOL_ID)\n"
	@printf "  Blend Token                  = $(BLEND_TOKEN_ID)\n"
	@printf "\n"
	@printf "$(GREEN)CoopStable Core Contracts:$(NC)\n"
	@printf "  CUSD Token                   = $(if $(CUSD_ID),$(CUSD_ID),$(RED)Not deployed$(NC))\n"
	@printf "  CUSD Manager                 = $(if $(CUSD_MANAGER_ID),$(CUSD_MANAGER_ID),$(RED)Not deployed$(NC))\n"
	@printf "  Yield Adapter Registry       = $(if $(YIELD_ADAPTER_REGISTRY_ID),$(YIELD_ADAPTER_REGISTRY_ID),$(RED)Not deployed$(NC))\n"
	@printf "  Yield Distributor            = $(if $(YIELD_DISTRIBUTOR_ID),$(YIELD_DISTRIBUTOR_ID),$(RED)Not deployed$(NC))\n"
	@printf "  Lending Yield Controller     = $(if $(LENDING_YIELD_CONTROLLER_ID),$(LENDING_YIELD_CONTROLLER_ID),$(RED)Not deployed$(NC))\n"
	@printf "\n"
	@printf "$(GREEN)Protocol Adapters:$(NC)\n"
	@printf "  Blend Capital Adapter        = $(if $(BLEND_CAPITAL_ADAPTER_ID),$(BLEND_CAPITAL_ADAPTER_ID),$(RED)Not deployed$(NC))\n"
	@printf "\n"
	@printf "$(GREEN)Account Keys:$(NC)\n"
	@printf "  Owner                        = $(OWNER)\n"
	@printf "  Admin                        = $(ADMIN)\n"
	@printf "  Treasury                     = $(TREASURY)\n"

.PHONY: save-addresses
save-addresses:
	@printf "$(YELLOW)Saving addresses...$(NC)\n"
	@echo "# Deployed contract addresses - $(shell date)" > deployed_addresses.mk
	@if [ ! -z "$(CUSD_ID)" ]; then echo "CUSD_ID = $(CUSD_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(CUSD_MANAGER_ID)" ]; then echo "CUSD_MANAGER_ID = $(CUSD_MANAGER_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(YIELD_ADAPTER_REGISTRY_ID)" ]; then echo "YIELD_ADAPTER_REGISTRY_ID = $(YIELD_ADAPTER_REGISTRY_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(YIELD_DISTRIBUTOR_ID)" ]; then echo "YIELD_DISTRIBUTOR_ID = $(YIELD_DISTRIBUTOR_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then echo "LENDING_YIELD_CONTROLLER_ID = $(LENDING_YIELD_CONTROLLER_ID)" >> deployed_addresses.mk; fi
	@if [ ! -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then echo "BLEND_CAPITAL_ADAPTER_ID = $(BLEND_CAPITAL_ADAPTER_ID)" >> deployed_addresses.mk; fi
	@echo "#!/bin/bash" > deployed_addresses.sh
	@echo "# Deployed contract addresses - $(shell date)" >> deployed_addresses.sh
	@if [ ! -z "$(CUSD_ID)" ]; then echo "export CUSD_ID=$(CUSD_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(CUSD_MANAGER_ID)" ]; then echo "export CUSD_MANAGER_ID=$(CUSD_MANAGER_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(YIELD_ADAPTER_REGISTRY_ID)" ]; then echo "export YIELD_ADAPTER_REGISTRY_ID=$(YIELD_ADAPTER_REGISTRY_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(YIELD_DISTRIBUTOR_ID)" ]; then echo "export YIELD_DISTRIBUTOR_ID=$(YIELD_DISTRIBUTOR_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(LENDING_YIELD_CONTROLLER_ID)" ]; then echo "export LENDING_YIELD_CONTROLLER_ID=$(LENDING_YIELD_CONTROLLER_ID)" >> deployed_addresses.sh; fi
	@if [ ! -z "$(BLEND_CAPITAL_ADAPTER_ID)" ]; then echo "export BLEND_CAPITAL_ADAPTER_ID=$(BLEND_CAPITAL_ADAPTER_ID)" >> deployed_addresses.sh; fi
	@chmod +x deployed_addresses.sh
	@printf "$(GREEN)Addresses saved to deployed_addresses.mk and deployed_addresses.sh$(NC)\n"


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
	@if [ ! -z "$(CUSD_ID)" ]; then \
		printf "$(GREEN)✓ CUSD Token: $(CUSD_ID)$(NC)\n"; \
	else \
		printf "$(RED)✗ CUSD Token not deployed$(NC)\n"; \
	fi
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

# ========== ADDITIONAL TESTING UTILITIES ==========

# distributor
.PHONY: add-member
add-member:
	@printf "$(YELLOW)Adding admin as community member for yield distribution...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		add_member \
		--member $(MEMBER)
	@printf "$(GREEN)Admin added as community member!$(NC)\n"

.PHONY: list-members
list-members:
	@printf "$(YELLOW)Query members...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		list_members 
	@printf "$(GREEN)Members listed!$(NC)\n"

.PHONY: next-distribution
next-distribution:
	@printf "$(YELLOW)Query next distribution time...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		get_next_distribution_time 
	@printf "$(GREEN)Next distribution time queried!$(NC)\n"

.PHONY: next-distribution-time-left
next-distribution-time-left:
	@printf "$(YELLOW)Query next distribution time left...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		time_before_next_distribution
	@printf "$(GREEN)Next distribution time left queried!$(NC)\n"

.PHONY: current-distribution
current-distribution:
	@printf "$(YELLOW)Query current distribution info...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		get_distribution_info 
	@printf "$(GREEN)Next distribution time queried!$(NC)\n"

.PHONY: distribution-history
distribution-history:
	@printf "$(YELLOW)Query distribution history...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		get_distribution_history 
	@printf "$(GREEN)Next distribution time queried!$(NC)\n"

.PHONY: check-distribution-status
check-distribution-status:
	@printf "$(YELLOW)Checking distribution status...$(NC)\n"
	@if [ -z "$(YIELD_DISTRIBUTOR_ID)" ]; then \
		printf "$(RED)Error: Yield Distributor ID not set.$(NC)\n"; \
		exit 1; \
	fi
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(YIELD_DISTRIBUTOR_ID) \
		-- \
		is_distribution_available 
	@printf "$(GREEN)Distribution status checked!$(NC)\n"

# cusd
.PHONY: cusd-check-balance
cusd-check-balance:
	@printf "$(YELLOW)Checking CUSD balance of $(ADMIN)...$(NC)\n"
	stellar contract invoke \
		--source $(ADMIN_KEY) \
		--network $(NETWORK) \
		--id $(CUSD_ID) \
		-- \
		balance \
		--account $(ACCOUNT) || printf "$(RED)Error reading CUSD balance$(NC)\n"

# Check protocol configuration status
.PHONY: check-config
check-config:
	@printf "$(YELLOW)Checking protocol configuration...$(NC)\n"
	@printf "$(GREEN)CUSD Manager yield controller:$(NC)\n"
	@if [ ! -z "$(CUSD_MANAGER_ID)" ]; then \
		stellar contract invoke \
			--source $(ADMIN_KEY) \
			--network $(NETWORK) \
			--id $(CUSD_MANAGER_ID) \
			-- \
			get_yield_controller || printf "$(RED)Error reading yield controller$(NC)\n"; \
	fi
	@printf "$(GREEN)Registry adapters:$(NC)\n"
	@if [ ! -z "$(YIELD_ADAPTER_REGISTRY_ID)" ]; then \
		stellar contract invoke \
			--source $(ADMIN_KEY) \
			--network $(NETWORK) \
			--id $(YIELD_ADAPTER_REGISTRY_ID) \
			-- \
			get_adapters_with_assets || printf "$(RED)Error reading adapters$(NC)\n"; \
	fi

# Clean deployment files
.PHONY: clean-deployment
clean-deployment:
	@printf "$(YELLOW)Cleaning deployment files...$(NC)\n"
	@rm -f deployed_addresses.mk deployed_addresses.sh
	@printf "$(GREEN)Deployment files cleaned!$(NC)\n"

# ========== REDEPLOY TARGETS ==========

.PHONY: redeploy-cusd-token
redeploy-cusd-token: deploy-cusd-token save-addresses
	@printf "$(GREEN)CUSD Token redeployed!$(NC)\n"

.PHONY: redeploy-cusd-manager
redeploy-cusd-manager: deploy-cusd-manager save-addresses
	@printf "$(GREEN)CUSD Manager redeployed!$(NC)\n"

.PHONY: redeploy-registry
redeploy-registry: deploy-registry save-addresses
	@printf "$(GREEN)Registry redeployed!$(NC)\n"

.PHONY: redeploy-distributor
redeploy-distributor: deploy-distributor save-addresses
	@printf "$(GREEN)Distributor redeployed!$(NC)\n"

.PHONY: redeploy-controller
redeploy-controller: deploy-controller save-addresses
	@printf "$(GREEN)Controller redeployed!$(NC)\n"

.PHONY: redeploy-blend-adapter
redeploy-blend-adapter: deploy-blend-adapter save-addresses
	@printf "$(GREEN)Blend adapter redeployed!$(NC)\n"

.DEFAULT_GOAL := help