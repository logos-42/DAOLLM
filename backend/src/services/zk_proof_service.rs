//! TRO ZK Proof Service
//!
//! Implements the proof layer of TRO pipeline:
//! - Reasoning trace hashing
//! - ZK proof generation (Risc0/SP1 compatible structure)
//! - Proof verification interface
//! - Proof registry and caching

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct ZKProofConfig {
    /// Enable async proof generation
    pub async_generation: bool,
    /// Proof generation timeout (seconds)
    pub generation_timeout_secs: u64,
    /// Maximum proof size (bytes)
    pub max_proof_size: usize,
    /// Cache generated proofs
    pub enable_cache: bool,
    /// Cache size limit
    pub cache_size: usize,
    /// ZK prover backend
    pub prover_backend: ProverBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProverBackend {
    /// Mock prover for development/testing
    Mock,
    /// Risc0 zkVM
    Risc0,
    /// SP1 (Succinct)
    SP1,
    /// Halo2 (PlonK)
    Halo2,
}

impl Default for ZKProofConfig {
    fn default() -> Self {
        Self {
            async_generation: true,
            generation_timeout_secs: 300,
            max_proof_size: 1024 * 1024, // 1MB
            enable_cache: true,
            cache_size: 1000,
            prover_backend: ProverBackend::Mock, // Default to mock for MVP
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Proof policy determining when proofs are required
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofPolicy {
    /// No proof required
    None,
    /// Proof optional (generates if resources available)
    Optional,
    /// Proof strongly recommended
    Recommended,
    /// Proof mandatory (fails without proof)
    Mandatory,
}

impl ProofPolicy {
    pub fn from_criticality(criticality: &str) -> Self {
        match criticality.to_lowercase().as_str() {
            "mission_critical" => ProofPolicy::Mandatory,
            "high" => ProofPolicy::Recommended,
            "medium" => ProofPolicy::Optional,
            _ => ProofPolicy::None,
        }
    }
}

/// Input for ZK proof generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofInput {
    pub task_id: u64,
    /// Original prompt/intent
    pub prompt_hash: [u8; 32],
    /// Model identifier
    pub model_id: String,
    /// Model capability tier
    pub model_capability: String,
    /// Workflow class
    pub workflow: String,
    /// Reasoning output hash
    pub output_hash: [u8; 32],
    /// Timestamp of inference
    pub timestamp: i64,
    /// Verification score (0-10000)
    pub verification_score: u16,
    /// Additional public inputs
    pub public_inputs: Vec<[u8; 32]>,
}

/// Generated ZK proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZKProof {
    pub proof_id: String,
    pub task_id: u64,
    /// The proof bytes (format depends on backend)
    pub proof_data: Vec<u8>,
    /// Public inputs that can be verified on-chain
    pub public_inputs: Vec<u8>,
    /// Proof type/circuit identifier
    pub proof_type: String,
    /// Prover backend used
    pub prover: String,
    /// Proof generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Proof size in bytes
    pub size_bytes: usize,
    /// Verification key hash (for on-chain verification)
    pub vk_hash: [u8; 32],
}

impl ZKProof {
    /// Compute the proof hash for on-chain storage
    pub fn proof_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.proof_data);
        hasher.update(&self.public_inputs);
        hasher.finalize().into()
    }
}

/// Proof verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerification {
    pub valid: bool,
    pub verification_time_ms: u64,
    pub error: Option<String>,
}

/// Proof generation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofStatus {
    Pending,
    Generating,
    Completed(ZKProof),
    Failed(String),
}

// ============================================================================
// ZK Proof Service
// ============================================================================

pub struct ZKProofService {
    config: ZKProofConfig,
    /// Proof cache
    cache: RwLock<HashMap<u64, ZKProof>>,
    /// Pending proof generations
    pending: RwLock<HashMap<u64, ProofStatus>>,
    /// Statistics
    stats: RwLock<ZKStats>,
}

#[derive(Debug, Clone, Default)]
pub struct ZKStats {
    pub proofs_generated: u64,
    pub proofs_verified: u64,
    pub proofs_failed: u64,
    pub avg_generation_time_ms: u64,
    pub avg_proof_size: usize,
}

impl ZKProofService {
    pub fn new(config: ZKProofConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
            pending: RwLock::new(HashMap::new()),
            stats: RwLock::new(ZKStats::default()),
        }
    }

    /// Generate a ZK proof for a reasoning task
    pub async fn generate_proof(&self, input: ProofInput) -> Result<ZKProof> {
        let start = std::time::Instant::now();
        info!("Generating ZK proof for task {}", input.task_id);

        // Check cache first
        if self.config.enable_cache {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&input.task_id) {
                debug!("Returning cached proof for task {}", input.task_id);
                return Ok(cached.clone());
            }
        }

        // Mark as generating
        {
            let mut pending = self.pending.write().await;
            pending.insert(input.task_id, ProofStatus::Generating);
        }

        // Generate proof based on backend
        let result = match self.config.prover_backend {
            ProverBackend::Mock => self.generate_mock_proof(&input).await,
            ProverBackend::Risc0 => self.generate_risc0_proof(&input).await,
            ProverBackend::SP1 => self.generate_sp1_proof(&input).await,
            ProverBackend::Halo2 => self.generate_halo2_proof(&input).await,
        };

        let generation_time = start.elapsed().as_millis() as u64;

        match result {
            Ok(proof) => {
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.proofs_generated += 1;
                    stats.avg_generation_time_ms = (stats.avg_generation_time_ms
                        * (stats.proofs_generated - 1)
                        + generation_time)
                        / stats.proofs_generated;
                    stats.avg_proof_size = (stats.avg_proof_size * (stats.proofs_generated - 1) as usize
                        + proof.size_bytes)
                        / stats.proofs_generated as usize;
                }

                // Cache proof
                if self.config.enable_cache {
                    let mut cache = self.cache.write().await;
                    if cache.len() >= self.config.cache_size {
                        // Evict oldest
                        if let Some(oldest_key) = cache.keys().next().cloned() {
                            cache.remove(&oldest_key);
                        }
                    }
                    cache.insert(input.task_id, proof.clone());
                }

                // Update pending status
                {
                    let mut pending = self.pending.write().await;
                    pending.insert(input.task_id, ProofStatus::Completed(proof.clone()));
                }

                info!(
                    "ZK proof generated for task {} in {}ms (size: {} bytes)",
                    input.task_id, generation_time, proof.size_bytes
                );

                Ok(proof)
            }
            Err(e) => {
                let mut stats = self.stats.write().await;
                stats.proofs_failed += 1;

                let mut pending = self.pending.write().await;
                pending.insert(input.task_id, ProofStatus::Failed(e.to_string()));

                Err(e)
            }
        }
    }

    /// Generate mock proof (for testing)
    async fn generate_mock_proof(&self, input: &ProofInput) -> Result<ZKProof> {
        // Simulate proof generation time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create deterministic mock proof
        let mut hasher = Sha256::new();
        hasher.update(&input.prompt_hash);
        hasher.update(&input.output_hash);
        hasher.update(input.model_id.as_bytes());
        hasher.update(&input.timestamp.to_le_bytes());

        let proof_seed: [u8; 32] = hasher.finalize().into();

        // Generate mock proof data (simulating ~1KB Groth16 proof)
        let mut proof_data = Vec::with_capacity(1024);
        for i in 0..32 {
            proof_data.extend_from_slice(&proof_seed);
            proof_data[i * 32] ^= i as u8;
        }

        // Generate public inputs (serialized for on-chain verification)
        let mut public_inputs = Vec::new();
        public_inputs.extend_from_slice(&input.prompt_hash);
        public_inputs.extend_from_slice(&input.output_hash);
        public_inputs.extend_from_slice(&input.timestamp.to_le_bytes());
        public_inputs.extend_from_slice(&input.verification_score.to_le_bytes());

        // Mock verification key hash
        let mut vk_hasher = Sha256::new();
        vk_hasher.update(b"mock_vk_");
        vk_hasher.update(input.model_id.as_bytes());
        let vk_hash: [u8; 32] = vk_hasher.finalize().into();

        Ok(ZKProof {
            proof_id: format!("mock_{}_{}", input.task_id, input.timestamp),
            task_id: input.task_id,
            proof_data: proof_data.clone(),
            public_inputs,
            proof_type: "mock_groth16".to_string(),
            prover: "mock".to_string(),
            generated_at: Utc::now(),
            size_bytes: proof_data.len(),
            vk_hash,
        })
    }

    /// Generate Risc0 proof (placeholder)
    async fn generate_risc0_proof(&self, input: &ProofInput) -> Result<ZKProof> {
        // In production, this would use risc0-zkvm
        // For MVP, fall back to mock
        warn!("Risc0 prover not implemented, using mock");
        self.generate_mock_proof(input).await
    }

    /// Generate SP1 proof (placeholder)
    async fn generate_sp1_proof(&self, input: &ProofInput) -> Result<ZKProof> {
        // In production, this would use SP1/Succinct SDK
        warn!("SP1 prover not implemented, using mock");
        self.generate_mock_proof(input).await
    }

    /// Generate Halo2 proof (placeholder)
    async fn generate_halo2_proof(&self, input: &ProofInput) -> Result<ZKProof> {
        // In production, this would use halo2 library
        warn!("Halo2 prover not implemented, using mock");
        self.generate_mock_proof(input).await
    }

    /// Verify a ZK proof
    pub async fn verify_proof(&self, proof: &ZKProof) -> ProofVerification {
        let start = std::time::Instant::now();

        let result = match self.config.prover_backend {
            ProverBackend::Mock => self.verify_mock_proof(proof),
            _ => {
                // For other backends, use mock verification for now
                self.verify_mock_proof(proof)
            }
        };

        let mut stats = self.stats.write().await;
        stats.proofs_verified += 1;

        ProofVerification {
            valid: result.is_ok(),
            verification_time_ms: start.elapsed().as_millis() as u64,
            error: result.err().map(|e| e.to_string()),
        }
    }

    /// Verify mock proof
    fn verify_mock_proof(&self, proof: &ZKProof) -> Result<()> {
        // Check proof structure
        if proof.proof_data.len() < 256 {
            return Err(anyhow!("Proof data too short"));
        }

        if proof.public_inputs.len() < 72 {
            return Err(anyhow!("Public inputs too short"));
        }

        // Check proof type
        if !proof.proof_type.starts_with("mock") {
            return Err(anyhow!("Unknown proof type"));
        }

        Ok(())
    }

    /// Get proof generation status
    pub async fn get_status(&self, task_id: u64) -> Option<ProofStatus> {
        let pending = self.pending.read().await;
        pending.get(&task_id).cloned()
    }

    /// Get cached proof
    pub async fn get_cached_proof(&self, task_id: u64) -> Option<ZKProof> {
        let cache = self.cache.read().await;
        cache.get(&task_id).cloned()
    }

    /// Get statistics
    pub async fn get_stats(&self) -> ZKStats {
        self.stats.read().await.clone()
    }

    /// Create proof input from reasoning trace
    pub fn create_proof_input(
        task_id: u64,
        prompt: &str,
        model_id: &str,
        model_capability: &str,
        workflow: &str,
        output: &str,
        verification_score: u16,
    ) -> ProofInput {
        let mut prompt_hasher = Sha256::new();
        prompt_hasher.update(prompt.as_bytes());
        let prompt_hash: [u8; 32] = prompt_hasher.finalize().into();

        let mut output_hasher = Sha256::new();
        output_hasher.update(output.as_bytes());
        let output_hash: [u8; 32] = output_hasher.finalize().into();

        ProofInput {
            task_id,
            prompt_hash,
            model_id: model_id.to_string(),
            model_capability: model_capability.to_string(),
            workflow: workflow.to_string(),
            output_hash,
            timestamp: Utc::now().timestamp(),
            verification_score,
            public_inputs: vec![],
        }
    }
}

// ============================================================================
// On-Chain Proof Registry Interface
// ============================================================================

/// Interface for on-chain proof verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainProofData {
    /// Proof hash (32 bytes)
    pub proof_hash: [u8; 32],
    /// Verification key hash
    pub vk_hash: [u8; 32],
    /// Compressed public inputs
    pub public_inputs_hash: [u8; 32],
    /// Proof policy used
    pub policy: ProofPolicy,
    /// Model capability
    pub model_capability: String,
    /// Workflow class
    pub workflow: String,
    /// Generation timestamp
    pub timestamp: i64,
}

impl OnChainProofData {
    /// Create on-chain data from a full proof
    pub fn from_proof(proof: &ZKProof, policy: ProofPolicy) -> Self {
        let mut inputs_hasher = Sha256::new();
        inputs_hasher.update(&proof.public_inputs);
        let public_inputs_hash: [u8; 32] = inputs_hasher.finalize().into();

        Self {
            proof_hash: proof.proof_hash(),
            vk_hash: proof.vk_hash,
            public_inputs_hash,
            policy,
            model_capability: String::new(), // Would be extracted from proof
            workflow: String::new(),
            timestamp: proof.generated_at.timestamp(),
        }
    }

    /// Serialize for on-chain storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128);
        bytes.extend_from_slice(&self.proof_hash);
        bytes.extend_from_slice(&self.vk_hash);
        bytes.extend_from_slice(&self.public_inputs_hash);
        bytes.push(self.policy as u8);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_proof_generation() {
        let service = ZKProofService::new(ZKProofConfig::default());

        let input = ZKProofService::create_proof_input(
            1,
            "What is 2+2?",
            "llama3.1:8b",
            "Local7B",
            "ExpressLocal",
            "4",
            9000,
        );

        let proof = service.generate_proof(input).await.unwrap();

        assert!(!proof.proof_data.is_empty());
        assert!(!proof.public_inputs.is_empty());
        assert_eq!(proof.task_id, 1);
    }

    #[tokio::test]
    async fn test_proof_verification() {
        let service = ZKProofService::new(ZKProofConfig::default());

        let input = ZKProofService::create_proof_input(
            2,
            "Test prompt",
            "test_model",
            "Local7B",
            "Standard",
            "Test output",
            8000,
        );

        let proof = service.generate_proof(input).await.unwrap();
        let verification = service.verify_proof(&proof).await;

        assert!(verification.valid);
    }

    #[tokio::test]
    async fn test_proof_caching() {
        let config = ZKProofConfig {
            enable_cache: true,
            cache_size: 10,
            ..Default::default()
        };
        let service = ZKProofService::new(config);

        let input = ZKProofService::create_proof_input(
            3,
            "Cache test",
            "model",
            "Local7B",
            "Standard",
            "Output",
            7500,
        );

        // First generation
        let _proof1 = service.generate_proof(input.clone()).await.unwrap();

        // Second should hit cache
        let cached = service.get_cached_proof(3).await;
        assert!(cached.is_some());
    }

    #[test]
    fn test_on_chain_data_serialization() {
        let proof = ZKProof {
            proof_id: "test".to_string(),
            task_id: 1,
            proof_data: vec![0u8; 256],
            public_inputs: vec![0u8; 72],
            proof_type: "mock_groth16".to_string(),
            prover: "mock".to_string(),
            generated_at: Utc::now(),
            size_bytes: 256,
            vk_hash: [0u8; 32],
        };

        let on_chain = OnChainProofData::from_proof(&proof, ProofPolicy::Mandatory);
        let bytes = on_chain.to_bytes();

        assert!(bytes.len() >= 97); // 32 + 32 + 32 + 1 + 8
    }

    #[test]
    fn test_proof_policy_from_criticality() {
        assert_eq!(
            ProofPolicy::from_criticality("mission_critical"),
            ProofPolicy::Mandatory
        );
        assert_eq!(ProofPolicy::from_criticality("high"), ProofPolicy::Recommended);
        assert_eq!(ProofPolicy::from_criticality("medium"), ProofPolicy::Optional);
        assert_eq!(ProofPolicy::from_criticality("low"), ProofPolicy::None);
    }
}

