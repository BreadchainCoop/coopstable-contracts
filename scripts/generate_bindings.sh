#!/bin/bash
# Set your network information
RPC_URL="http://localhost:8000"
NETWORK_PASSPHRASE="Standalone Network ; February 2017"
NETWORK="Standalone"

# Contract ID - replace with your actual contract ID or use a different ID for each contract
CONTRACT_ID="CBWH54OKUK6U2J2A4J2REJEYB625NEFCHISWXLOPR2D2D6FTN63TJTWN"

# Directories
TARGET_DIR="./target/wasm32-unknown-unknown/release"
OUTPUT_BASE_DIR="./ts"

# Create output directory if it doesn't exist
mkdir -p $OUTPUT_BASE_DIR

# Build all contracts first
echo "Building all contracts..."
make build

# List of contracts from your project structure
CONTRACTS=(
    "cusd_manager"
    "yield_adapter_registry"
    "yield_distributor"
    "lending_yield_controller"
)

# Packages (adapters)
PACKAGES=(
    "access_control"
    "blend_capital_adapter"
    "yield_adapter"
)

# Generate bindings for specific contracts
generate_contract_bindings() {
    for contract in "${CONTRACTS[@]}"; do
        wasm_path="${TARGET_DIR}/${contract}.wasm"
        output_dir="${OUTPUT_BASE_DIR}/ts-${contract}"
        
        if [ -f "$wasm_path" ]; then
            echo "Generating TypeScript bindings for ${contract}..."
            
            mkdir -p $output_dir
            
            stellar contract bindings typescript --overwrite \
                --contract-id $CONTRACT_ID \
                --wasm $wasm_path \
                --output-dir $output_dir \
                --rpc-url $RPC_URL \
                --network-passphrase "$NETWORK_PASSPHRASE" \
                --network $NETWORK
                
            echo "Generated bindings for ${contract} in ${output_dir}"
        else
            echo "Warning: WASM file not found for ${contract}: ${wasm_path}"
        fi
    done
}

# Generate bindings for packages/adapters
generate_package_bindings() {
    for package in "${PACKAGES[@]}"; do
        wasm_path="${TARGET_DIR}/${package}.wasm"
        output_dir="${OUTPUT_BASE_DIR}/ts-${package}"
        
        if [ -f "$wasm_path" ]; then
            echo "Generating TypeScript bindings for ${package}..."
            
            mkdir -p $output_dir
            
            stellar contract bindings typescript --overwrite \
                --contract-id $CONTRACT_ID \
                --wasm $wasm_path \
                --output-dir $output_dir \
                --rpc-url $RPC_URL \
                --network-passphrase "$NETWORK_PASSPHRASE" \
                --network $NETWORK
                
            echo "Generated bindings for ${package} in ${output_dir}"
        else
            echo "Warning: WASM file not found for ${package}: ${wasm_path}"
        fi
    done
}

# Generate bindings for other WASM files (if any)
generate_other_bindings() {
    for wasm_file in $(find $TARGET_DIR -name "*.wasm"); do
        contract_name=$(basename $wasm_file .wasm)
        
        # Skip already processed contracts and packages
        if [[ " ${CONTRACTS[@]} ${PACKAGES[@]} " =~ " ${contract_name} " ]]; then
            continue
        fi
        
        output_dir="${OUTPUT_BASE_DIR}/ts-${contract_name}"
        
        echo "Generating TypeScript bindings for ${contract_name}..."
        
        mkdir -p $output_dir
        
        stellar contract bindings typescript --overwrite \
            --contract-id $CONTRACT_ID \
            --wasm $wasm_file \
            --output-dir $output_dir \
            --rpc-url $RPC_URL \
            --network-passphrase "$NETWORK_PASSPHRASE" \
            --network $NETWORK
            
        echo "Generated bindings for ${contract_name} in ${output_dir}"
    done
}

# Generate all bindings
echo "Generating bindings for contracts..."
generate_contract_bindings

echo "Generating bindings for packages/adapters..."
generate_package_bindings

echo "Generating bindings for any remaining WASM files..."
generate_other_bindings

echo "Bindings generation complete!"