#!/bin/bash

# Environment Setup Script

set -e

echo "🔧 Setting up development environment..."

# Check Rust
if ! command -v rustc &> /dev/null; then
    echo "📦 Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
fi

# Check Solana CLI
if ! command -v solana &> /dev/null; then
    echo "📦 Installing Solana CLI..."
    sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
fi

# Check Anchor
if ! command -v anchor &> /dev/null; then
    echo "📦 Installing Anchor..."
    cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
    avm install latest
    avm use latest
fi

# Check Node.js
if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Please install Node.js 18+ first."
    exit 1
fi

# Setup Solana
echo "⚙️  Configuring Solana..."
solana config set --url devnet

# Create keypair if doesn't exist
if [ ! -f ~/.config/solana/id.json ]; then
    echo "🔑 Generating Solana keypair..."
    solana-keygen new
fi

# Airdrop test SOL
echo "💰 Requesting test SOL..."
solana airdrop 2

echo "✅ Setup complete!"

