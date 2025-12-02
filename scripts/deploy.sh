#!/bin/bash

# DAO Proposal System Deployment Script

set -e

echo "🚀 Starting deployment..."

# Check if Solana CLI is installed
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not found. Please install it first."
    exit 1
fi

# Check if Anchor is installed
if ! command -v anchor &> /dev/null; then
    echo "❌ Anchor not found. Please install it first."
    exit 1
fi

# Set network
NETWORK=${1:-devnet}
echo "📡 Using network: $NETWORK"

# Configure Solana CLI
solana config set --url $NETWORK

# Build the program
echo "🔨 Building program..."
anchor build

# Deploy the program
echo "📤 Deploying program..."
anchor deploy

# Get program ID
PROGRAM_ID=$(solana address -k target/deploy/daollm-keypair.json)
echo "✅ Program deployed with ID: $PROGRAM_ID"

# Save program ID to .env
if [ -f .env ]; then
    sed -i.bak "s/PROGRAM_ID=.*/PROGRAM_ID=$PROGRAM_ID/" .env
else
    echo "PROGRAM_ID=$PROGRAM_ID" >> .env
fi

echo "🎉 Deployment complete!"
echo "Program ID: $PROGRAM_ID"

