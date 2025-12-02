#!/bin/bash

# Test Script for DAO Proposal System

set -e

echo "🧪 Running tests..."

# Test Solana program
echo "📝 Testing Solana program..."
anchor test

# Test Rust backend (if tests exist)
if [ -d "backend/tests" ]; then
    echo "🔧 Testing Rust backend..."
    cd backend
    cargo test
    cd ..
fi

echo "✅ All tests passed!"

