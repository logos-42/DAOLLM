# TRO Implementation Verification Script
# Checks that all planned components are implemented

Write-Host "🔍 TRO Implementation Verification" -ForegroundColor Cyan
Write-Host "===================================`n" -ForegroundColor Cyan

$errors = 0
$success = 0

function Test-File {
    param($path, $description)
    if (Test-Path $path) {
        Write-Host "✅ $description" -ForegroundColor Green
        Write-Host "   $path" -ForegroundColor Gray
        $script:success++
        return $true
    } else {
        Write-Host "❌ $description" -ForegroundColor Red
        Write-Host "   Missing: $path" -ForegroundColor Yellow
        $script:errors++
        return $false
    }
}

Write-Host "📦 Smart Contract (Solana/Anchor):" -ForegroundColor Yellow
Test-File "programs/daollm/src/state/tro.rs" "TRO State Structures"
Test-File "programs/daollm/src/instructions/tro.rs" "TRO Instructions"
Test-File "programs/daollm/src/state/node.rs" "ReasoningNode Extension"
Write-Host ""

Write-Host "🔧 Backend Services (Rust):" -ForegroundColor Yellow
Test-File "backend/src/services/reasoning_service.rs" "Reasoning Service"
Test-File "backend/src/services/semantic_cache_service.rs" "Semantic Cache Service"
Test-File "backend/src/services/prompt_optimizer.rs" "Prompt Optimizer"
Test-File "backend/src/services/knowledge_graph_service.rs" "Knowledge Graph Service"
Test-File "backend/src/services/verification_service.rs" "Verification Service"
Test-File "backend/src/services/zk_proof_service.rs" "ZK Proof Service"
Test-File "backend/src/services/ipfs_service.rs" "Enhanced IPFS Service"
Write-Host ""

Write-Host "🖥️ Frontend Pages (Next.js):" -ForegroundColor Yellow
Test-File "frontend/src/pages/tro-tasks.tsx" "TRO Tasks Page"
Test-File "frontend/src/pages/task-monitor.tsx" "Task Monitor Page"
Test-File "frontend/src/pages/node-register.tsx" "Node Register Page"
Test-File "frontend/src/pages/challenge.tsx" "Challenge/Dispute Page"
Write-Host ""

Write-Host "📊 Tests & Config:" -ForegroundColor Yellow
Test-File "backend/tests/tro_benchmark.rs" "Performance Benchmark Tests"
Test-File "scripts/deploy-devnet.sh" "Devnet Deployment Script"
Test-File "config/economy-params.json" "Economy Model Configuration"
Write-Host ""

Write-Host "===================================" -ForegroundColor Cyan
Write-Host "Summary:" -ForegroundColor Cyan
Write-Host "  ✅ Success: $success" -ForegroundColor Green
Write-Host "  ❌ Errors: $errors" -ForegroundColor $(if ($errors -eq 0) { "Green" } else { "Red" })
Write-Host ""

if ($errors -eq 0) {
    Write-Host "🎉 All TRO components verified!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "  1. Install dependencies: cd backend && cargo build" -ForegroundColor White
    Write-Host "  2. Build frontend: cd frontend && npm install" -ForegroundColor White
    Write-Host "  3. Deploy to devnet: ./scripts/deploy-devnet.sh" -ForegroundColor White
    exit 0
} else {
    Write-Host "⚠️  Some components are missing. Please check the errors above." -ForegroundColor Yellow
    exit 1
}

