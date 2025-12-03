#!/bin/bash
# TRO Deployment Script for Solana Devnet
# 
# Prerequisites:
# - Solana CLI installed (see docs/SOLANA_MANUAL_INSTALL.md)
# - Anchor CLI installed
# - Node.js 18+ for frontend
# - Rust 1.70+ for backend

set -e

echo "🚀 TRO Devnet Deployment Script"
echo "================================"

# Configuration
CLUSTER="devnet"
PROGRAM_NAME="daollm"
BACKEND_PORT=8080
FRONTEND_PORT=3000

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
check_prerequisites() {
    echo -e "\n${YELLOW}Checking prerequisites...${NC}"
    
    if ! command -v solana &> /dev/null; then
        echo -e "${RED}Error: Solana CLI not found${NC}"
        echo "Please install Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools"
        exit 1
    fi
    
    if ! command -v anchor &> /dev/null; then
        echo -e "${RED}Error: Anchor CLI not found${NC}"
        echo "Please install Anchor: cargo install --git https://github.com/coral-xyz/anchor anchor-cli"
        exit 1
    fi
    
    if ! command -v node &> /dev/null; then
        echo -e "${RED}Error: Node.js not found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}All prerequisites satisfied${NC}"
}

# Configure Solana for devnet
configure_solana() {
    echo -e "\n${YELLOW}Configuring Solana for devnet...${NC}"
    
    solana config set --url https://api.devnet.solana.com
    
    # Check wallet balance
    BALANCE=$(solana balance 2>/dev/null || echo "0")
    echo "Current wallet balance: $BALANCE"
    
    if [[ "$BALANCE" == "0" || "$BALANCE" == "0 SOL" ]]; then
        echo -e "${YELLOW}Requesting airdrop...${NC}"
        solana airdrop 2 || echo "Airdrop may have failed, please try manually"
    fi
    
    echo -e "${GREEN}Solana configured for devnet${NC}"
}

# Build the Anchor program
build_program() {
    echo -e "\n${YELLOW}Building Anchor program...${NC}"
    
    cd programs/daollm
    
    # Build
    anchor build
    
    # Get program ID
    PROGRAM_ID=$(solana address -k target/deploy/${PROGRAM_NAME}-keypair.json 2>/dev/null || echo "")
    
    if [ -n "$PROGRAM_ID" ]; then
        echo -e "${GREEN}Program built successfully${NC}"
        echo "Program ID: $PROGRAM_ID"
        
        # Update lib.rs with new program ID
        sed -i "s/declare_id!(\"[^\"]*\")/declare_id!(\"$PROGRAM_ID\")/" src/lib.rs
    fi
    
    cd ../..
}

# Deploy to devnet
deploy_program() {
    echo -e "\n${YELLOW}Deploying to devnet...${NC}"
    
    cd programs/daollm
    
    # Deploy
    anchor deploy --provider.cluster devnet
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}Program deployed successfully!${NC}"
        
        # Save deployment info
        PROGRAM_ID=$(solana address -k target/deploy/${PROGRAM_NAME}-keypair.json)
        DEPLOY_TIME=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        
        cat > ../../deployments/devnet.json << EOF
{
    "cluster": "devnet",
    "programId": "$PROGRAM_ID",
    "deployedAt": "$DEPLOY_TIME",
    "rpcUrl": "https://api.devnet.solana.com"
}
EOF
        echo "Deployment info saved to deployments/devnet.json"
    else
        echo -e "${RED}Deployment failed${NC}"
        exit 1
    fi
    
    cd ../..
}

# Build and start backend
setup_backend() {
    echo -e "\n${YELLOW}Setting up backend...${NC}"
    
    cd backend
    
    # Create .env if not exists
    if [ ! -f .env ]; then
        cat > .env << EOF
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=$BACKEND_PORT
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=$(cat ../deployments/devnet.json 2>/dev/null | grep programId | cut -d'"' -f4 || echo "")
IPFS_API_URL=http://localhost:5001
PINATA_API_KEY=
PINATA_SECRET_KEY=
PINATA_GATEWAY_URL=https://gateway.pinata.cloud/ipfs/
REDIS_URL=redis://127.0.0.1:6379
OLLAMA_ENDPOINT=http://localhost:11434
EOF
        echo "Created .env file"
    fi
    
    # Build backend
    cargo build --release
    
    echo -e "${GREEN}Backend built successfully${NC}"
    
    cd ..
}

# Setup frontend
setup_frontend() {
    echo -e "\n${YELLOW}Setting up frontend...${NC}"
    
    cd frontend
    
    # Install dependencies
    npm install
    
    # Create .env.local if not exists
    if [ ! -f .env.local ]; then
        PROGRAM_ID=$(cat ../deployments/devnet.json 2>/dev/null | grep programId | cut -d'"' -f4 || echo "")
        cat > .env.local << EOF
NEXT_PUBLIC_SOLANA_RPC_URL=https://api.devnet.solana.com
NEXT_PUBLIC_PROGRAM_ID=$PROGRAM_ID
NEXT_PUBLIC_BACKEND_URL=http://localhost:$BACKEND_PORT
EOF
        echo "Created .env.local file"
    fi
    
    # Build frontend
    npm run build
    
    echo -e "${GREEN}Frontend built successfully${NC}"
    
    cd ..
}

# Run integration tests
run_tests() {
    echo -e "\n${YELLOW}Running integration tests...${NC}"
    
    cd programs/daollm
    anchor test --provider.cluster devnet
    cd ../..
    
    echo -e "${GREEN}Tests completed${NC}"
}

# Print summary
print_summary() {
    echo -e "\n${GREEN}================================${NC}"
    echo -e "${GREEN}Deployment Complete!${NC}"
    echo -e "${GREEN}================================${NC}"
    
    if [ -f deployments/devnet.json ]; then
        echo ""
        echo "Deployment Info:"
        cat deployments/devnet.json
    fi
    
    echo ""
    echo "Next Steps:"
    echo "1. Start Redis: redis-server"
    echo "2. Start Ollama: ollama serve"
    echo "3. Start Backend: cd backend && cargo run --release"
    echo "4. Start Frontend: cd frontend && npm run dev"
    echo ""
    echo "Access the app at: http://localhost:$FRONTEND_PORT"
}

# Main execution
main() {
    # Create deployments directory
    mkdir -p deployments
    
    # Run deployment steps
    check_prerequisites
    configure_solana
    build_program
    deploy_program
    setup_backend
    setup_frontend
    
    # Optionally run tests
    if [ "$1" == "--with-tests" ]; then
        run_tests
    fi
    
    print_summary
}

# Run main function
main "$@"

