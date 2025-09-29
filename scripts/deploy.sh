#!/bin/bash

# Script de deployment para contratos PermaMuseum
# Asegúrate de tener soroban-cli instalado y configurado

echo "🚀 Desplegando contratos PermaMuseum..."

# Compilar todos los contratos
echo "📦 Compilando contratos..."
cargo build --workspace --release

# Deploy MuseumRegistry
echo "🏛️ Desplegando MuseumRegistry..."
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/museum_registry.wasm \
    --source-account admin \
    --network testnet

# Deploy CulturalNFT  
echo "🎨 Desplegando CulturalNFT..."
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/cultural_nft.wasm \
    --source-account admin \
    --network testnet
    

# Deploy Marketplace
echo "🛒 Desplegando Marketplace..."
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/marketplace.wasm \
    --source-account admin \
    --network testnet

# Deploy SocialFi
echo "🏆 Desplegando SocialFi..."
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/socialfi.wasm \
    --source-account admin \
    --network testnet

echo "✅ Deployment completado!"
