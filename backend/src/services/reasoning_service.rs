//! TRO Reasoning Layer Service
//!
//! Implements the first stage of the TRO pipeline: Reasoning
//! - Local quantized LLM inference (7B/13B models via Ollama)
//! - Intelligent routing based on task complexity
//! - Request batching for GPU efficiency
//! - Streaming inference support
//! - Performance consensus testing

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

// ============================================================================
// Configuration
// ============================================================================

/// Model capability tiers matching on-chain ModelCapability enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    /// 7B parameter models (Llama-3.1-8B, Mistral-7B)
    Local7B,
    /// 13B parameter models
    Local13B,
    /// 70B+ parameter models (requires high-end GPU)
    Local70B,
    /// External API (OpenAI, Anthropic, etc.)
    CloudAPI,
}

impl ModelTier {
    pub fn default_model_name(&self) -> &'static str {
        match self {
            ModelTier::Local7B => "llama3.1:8b-instruct-q4_K_M",
            ModelTier::Local13B => "llama3.1:13b-instruct-q4_K_M",
            ModelTier::Local70B => "llama3.1:70b-instruct-q4_K_M",
            ModelTier::CloudAPI => "gpt-4o-mini",
        }
    }

    pub fn complexity_threshold(&self) -> u16 {
        match self {
            ModelTier::Local7B => 3000,   // Simple QA, factual lookups
            ModelTier::Local13B => 6000,  // Moderate reasoning
            ModelTier::Local70B => 8500,  // Complex multi-step
            ModelTier::CloudAPI => 10000, // Mission-critical
        }
    }
}

/// Workflow classes for routing decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowClass {
    /// Fast path: cache-first, local 7B fallback
    ExpressLocal,
    /// Standard: local 13B with verification
    Standard,
    /// High-precision: 70B or API with full verification + ZK
    HighPrecision,
    /// Critical: API-only with mandatory ZK proof
    MissionCritical,
}

/// Reasoning service configuration
#[derive(Debug, Clone)]
pub struct ReasoningConfig {
    /// Ollama API endpoint for local models
    pub ollama_endpoint: String,
    /// OpenAI-compatible API endpoint for cloud models
    pub cloud_api_endpoint: Option<String>,
    /// Cloud API key
    pub cloud_api_key: Option<String>,
    /// Maximum concurrent local inferences
    pub max_concurrent_local: usize,
    /// Maximum concurrent cloud API calls
    pub max_concurrent_cloud: usize,
    /// Request batch size for GPU efficiency
    pub batch_size: usize,
    /// Batch accumulation timeout (ms)
    pub batch_timeout_ms: u64,
    /// Inference timeout per request
    pub inference_timeout: Duration,
    /// Enable performance consensus testing
    pub enable_benchmark: bool,
    /// Benchmark interval (seconds)
    pub benchmark_interval_secs: u64,
}

impl Default for ReasoningConfig {
    fn default() -> Self {
        Self {
            ollama_endpoint: "http://localhost:11434".to_string(),
            cloud_api_endpoint: None,
            cloud_api_key: None,
            max_concurrent_local: 4,
            max_concurrent_cloud: 10,
            batch_size: 8,
            batch_timeout_ms: 100,
            inference_timeout: Duration::from_secs(120),
            enable_benchmark: true,
            benchmark_interval_secs: 3600, // 1 hour
        }
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// Reasoning request from TRO pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningRequest {
    pub task_id: u64,
    pub intent: String,
    pub context: Option<String>,
    pub complexity_score: u16,
    pub workflow: WorkflowClass,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    /// If true, bypass cache and force fresh inference
    pub force_fresh: bool,
    /// Metadata for tracing
    pub metadata: HashMap<String, String>,
}

/// Reasoning response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningResponse {
    pub task_id: u64,
    pub result: String,
    pub result_hash: String,
    pub model_used: String,
    pub model_tier: ModelTier,
    pub tokens_used: u32,
    pub inference_time_ms: u64,
    pub cache_hit: bool,
    pub confidence_bps: u16,
    pub timestamp: DateTime<Utc>,
    /// Reasoning trace for ZK proof generation
    pub trace_hash: String,
}

/// Ollama API request format
#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
    top_p: f32,
    seed: i32,
}

/// Ollama API response format
#[derive(Debug, Deserialize)]
struct OllamaResponse {
    model: String,
    response: String,
    done: bool,
    total_duration: Option<u64>,
    eval_count: Option<u32>,
}

/// OpenAI-compatible API request
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: u32,
}

// ============================================================================
// Model Router
// ============================================================================

/// Routing decision for a reasoning request
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub tier: ModelTier,
    pub model_name: String,
    pub reason: String,
}

/// Model router: decides which model tier to use based on task properties
pub struct ModelRouter {
    /// Routing strategy matrix: (workflow, complexity_range) -> tier
    strategy_matrix: HashMap<(WorkflowClass, u8), ModelTier>,
    /// Model availability status
    model_status: RwLock<HashMap<ModelTier, bool>>,
    /// Benchmark scores for each model (higher is better)
    benchmark_scores: RwLock<HashMap<ModelTier, f64>>,
}

impl ModelRouter {
    pub fn new() -> Self {
        let mut strategy_matrix = HashMap::new();

        // ExpressLocal workflow: prefer local models
        strategy_matrix.insert((WorkflowClass::ExpressLocal, 0), ModelTier::Local7B);
        strategy_matrix.insert((WorkflowClass::ExpressLocal, 1), ModelTier::Local7B);
        strategy_matrix.insert((WorkflowClass::ExpressLocal, 2), ModelTier::Local13B);
        strategy_matrix.insert((WorkflowClass::ExpressLocal, 3), ModelTier::Local13B);

        // Standard workflow: balanced approach
        strategy_matrix.insert((WorkflowClass::Standard, 0), ModelTier::Local7B);
        strategy_matrix.insert((WorkflowClass::Standard, 1), ModelTier::Local13B);
        strategy_matrix.insert((WorkflowClass::Standard, 2), ModelTier::Local13B);
        strategy_matrix.insert((WorkflowClass::Standard, 3), ModelTier::Local70B);

        // HighPrecision workflow: prefer larger models
        strategy_matrix.insert((WorkflowClass::HighPrecision, 0), ModelTier::Local13B);
        strategy_matrix.insert((WorkflowClass::HighPrecision, 1), ModelTier::Local70B);
        strategy_matrix.insert((WorkflowClass::HighPrecision, 2), ModelTier::Local70B);
        strategy_matrix.insert((WorkflowClass::HighPrecision, 3), ModelTier::CloudAPI);

        // MissionCritical workflow: always use cloud API
        strategy_matrix.insert((WorkflowClass::MissionCritical, 0), ModelTier::CloudAPI);
        strategy_matrix.insert((WorkflowClass::MissionCritical, 1), ModelTier::CloudAPI);
        strategy_matrix.insert((WorkflowClass::MissionCritical, 2), ModelTier::CloudAPI);
        strategy_matrix.insert((WorkflowClass::MissionCritical, 3), ModelTier::CloudAPI);

        Self {
            strategy_matrix,
            model_status: RwLock::new(HashMap::new()),
            benchmark_scores: RwLock::new(HashMap::new()),
        }
    }

    /// Route a request to the appropriate model tier
    pub async fn route(&self, request: &ReasoningRequest) -> RoutingDecision {
        // Map complexity score to bucket (0-3)
        let complexity_bucket = match request.complexity_score {
            0..=2500 => 0,
            2501..=5000 => 1,
            5001..=7500 => 2,
            _ => 3,
        };

        // Get base tier from strategy matrix
        let base_tier = self
            .strategy_matrix
            .get(&(request.workflow, complexity_bucket))
            .copied()
            .unwrap_or(ModelTier::Local13B);

        // Check model availability and fallback if needed
        let status = self.model_status.read().await;
        let tier = if status.get(&base_tier).copied().unwrap_or(true) {
            base_tier
        } else {
            // Fallback chain: Local7B -> Local13B -> Local70B -> CloudAPI
            self.find_fallback(base_tier, &status)
        };

        let model_name = tier.default_model_name().to_string();
        let reason = format!(
            "workflow={:?}, complexity={}, bucket={}, tier={:?}",
            request.workflow, request.complexity_score, complexity_bucket, tier
        );

        RoutingDecision {
            tier,
            model_name,
            reason,
        }
    }

    fn find_fallback(&self, preferred: ModelTier, status: &HashMap<ModelTier, bool>) -> ModelTier {
        let fallback_order = [
            ModelTier::Local7B,
            ModelTier::Local13B,
            ModelTier::Local70B,
            ModelTier::CloudAPI,
        ];

        for tier in fallback_order {
            if tier != preferred && status.get(&tier).copied().unwrap_or(true) {
                return tier;
            }
        }

        // Last resort: return preferred tier anyway
        preferred
    }

    /// Update model availability status
    pub async fn set_model_status(&self, tier: ModelTier, available: bool) {
        let mut status = self.model_status.write().await;
        status.insert(tier, available);
    }

    /// Update benchmark scores
    pub async fn update_benchmark(&self, tier: ModelTier, score: f64) {
        let mut scores = self.benchmark_scores.write().await;
        scores.insert(tier, score);
        info!("Updated benchmark score for {:?}: {:.2}", tier, score);
    }
}

// ============================================================================
// Reasoning Service
// ============================================================================

/// Main reasoning service implementing the TRO reasoning layer
pub struct ReasoningService {
    config: ReasoningConfig,
    client: Client,
    router: Arc<ModelRouter>,
    /// Pending requests for batching
    pending_requests: Mutex<Vec<(ReasoningRequest, mpsc::Sender<Result<ReasoningResponse>>)>>,
    /// Active inference count per tier
    active_inferences: RwLock<HashMap<ModelTier, usize>>,
    /// Metrics
    total_requests: Mutex<u64>,
    cache_hits: Mutex<u64>,
}

impl ReasoningService {
    pub fn new(config: ReasoningConfig) -> Self {
        let client = Client::builder()
            .timeout(config.inference_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            client,
            router: Arc::new(ModelRouter::new()),
            pending_requests: Mutex::new(Vec::new()),
            active_inferences: RwLock::new(HashMap::new()),
            total_requests: Mutex::new(0),
            cache_hits: Mutex::new(0),
        }
    }

    /// Process a reasoning request
    pub async fn process(&self, request: ReasoningRequest) -> Result<ReasoningResponse> {
        let start = std::time::Instant::now();
        *self.total_requests.lock().await += 1;

        // Get routing decision
        let routing = self.router.route(&request).await;
        debug!(
            "Routing task {} to {:?}: {}",
            request.task_id, routing.tier, routing.reason
        );

        // Execute inference based on tier
        let result = match routing.tier {
            ModelTier::Local7B | ModelTier::Local13B | ModelTier::Local70B => {
                self.inference_local(&request, &routing).await
            }
            ModelTier::CloudAPI => self.inference_cloud(&request, &routing).await,
        };

        match &result {
            Ok(response) => {
                info!(
                    "Task {} completed in {}ms using {} (cache_hit={})",
                    request.task_id,
                    start.elapsed().as_millis(),
                    response.model_used,
                    response.cache_hit
                );
            }
            Err(e) => {
                error!("Task {} failed: {}", request.task_id, e);
            }
        }

        result
    }

    /// Local inference via Ollama
    async fn inference_local(
        &self,
        request: &ReasoningRequest,
        routing: &RoutingDecision,
    ) -> Result<ReasoningResponse> {
        let start = std::time::Instant::now();

        // Build prompt
        let prompt = self.build_prompt(request);

        // Create Ollama request
        let ollama_req = OllamaRequest {
            model: routing.model_name.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: request.temperature.unwrap_or(0.1),
                num_predict: request.max_tokens.unwrap_or(2048) as i32,
                top_p: 0.9,
                seed: 42, // Deterministic for reproducibility
            },
        };

        // Send request to Ollama
        let url = format!("{}/api/generate", self.config.ollama_endpoint);
        let response = self
            .client
            .post(&url)
            .json(&ollama_req)
            .send()
            .await
            .map_err(|e| anyhow!("Ollama request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama error {}: {}", status, body));
        }

        let ollama_resp: OllamaResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        let inference_time_ms = start.elapsed().as_millis() as u64;

        // Compute hashes for verification
        let result_hash = self.compute_hash(&ollama_resp.response);
        let trace_hash = self.compute_trace_hash(request, &ollama_resp.response, &routing.model_name);

        // Estimate confidence based on model tier and response quality
        let confidence_bps = self.estimate_confidence(routing.tier, &ollama_resp.response);

        Ok(ReasoningResponse {
            task_id: request.task_id,
            result: ollama_resp.response,
            result_hash,
            model_used: routing.model_name.clone(),
            model_tier: routing.tier,
            tokens_used: ollama_resp.eval_count.unwrap_or(0),
            inference_time_ms,
            cache_hit: false,
            confidence_bps,
            timestamp: Utc::now(),
            trace_hash,
        })
    }

    /// Cloud API inference (OpenAI-compatible)
    async fn inference_cloud(
        &self,
        request: &ReasoningRequest,
        routing: &RoutingDecision,
    ) -> Result<ReasoningResponse> {
        let start = std::time::Instant::now();

        let endpoint = self
            .config
            .cloud_api_endpoint
            .as_ref()
            .ok_or_else(|| anyhow!("Cloud API endpoint not configured"))?;
        let api_key = self
            .config
            .cloud_api_key
            .as_ref()
            .ok_or_else(|| anyhow!("Cloud API key not configured"))?;

        // Build messages
        let messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: "You are a precise and reliable reasoning assistant. Provide accurate, well-structured responses.".to_string(),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: self.build_prompt(request),
            },
        ];

        let openai_req = OpenAIRequest {
            model: routing.model_name.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", endpoint))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| anyhow!("Cloud API request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Cloud API error {}: {}", status, body));
        }

        let openai_resp: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Cloud API response: {}", e))?;

        let result = openai_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        let inference_time_ms = start.elapsed().as_millis() as u64;
        let tokens_used = openai_resp.usage.map(|u| u.total_tokens).unwrap_or(0);

        let result_hash = self.compute_hash(&result);
        let trace_hash = self.compute_trace_hash(request, &result, &routing.model_name);
        let confidence_bps = self.estimate_confidence(routing.tier, &result);

        Ok(ReasoningResponse {
            task_id: request.task_id,
            result,
            result_hash,
            model_used: routing.model_name.clone(),
            model_tier: routing.tier,
            tokens_used,
            inference_time_ms,
            cache_hit: false,
            confidence_bps,
            timestamp: Utc::now(),
            trace_hash,
        })
    }

    /// Build the prompt for inference
    fn build_prompt(&self, request: &ReasoningRequest) -> String {
        let mut prompt = request.intent.clone();

        if let Some(ctx) = &request.context {
            prompt = format!("Context:\n{}\n\nTask:\n{}", ctx, prompt);
        }

        prompt
    }

    /// Compute SHA256 hash of content
    fn compute_hash(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        base64::encode(result)
    }

    /// Compute trace hash for ZK proof generation
    fn compute_trace_hash(&self, request: &ReasoningRequest, result: &str, model: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(request.intent.as_bytes());
        hasher.update(model.as_bytes());
        hasher.update(result.as_bytes());
        hasher.update(&request.task_id.to_le_bytes());
        let result = hasher.finalize();
        base64::encode(result)
    }

    /// Estimate confidence based on model tier and response quality
    fn estimate_confidence(&self, tier: ModelTier, response: &str) -> u16 {
        // Base confidence by tier
        let base = match tier {
            ModelTier::Local7B => 6000,
            ModelTier::Local13B => 7500,
            ModelTier::Local70B => 8500,
            ModelTier::CloudAPI => 9500,
        };

        // Adjust based on response quality heuristics
        let mut confidence = base;

        // Penalize very short responses
        if response.len() < 50 {
            confidence = confidence.saturating_sub(1000);
        }

        // Penalize responses that seem uncertain
        let uncertain_phrases = ["I'm not sure", "I don't know", "possibly", "might be"];
        for phrase in uncertain_phrases {
            if response.to_lowercase().contains(phrase) {
                confidence = confidence.saturating_sub(500);
            }
        }

        confidence.min(10000)
    }

    /// Get service statistics
    pub async fn get_stats(&self) -> ReasoningStats {
        let total = *self.total_requests.lock().await;
        let cache_hits = *self.cache_hits.lock().await;

        ReasoningStats {
            total_requests: total,
            cache_hits,
            cache_hit_rate: if total > 0 {
                (cache_hits as f64 / total as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Check if Ollama is available
    pub async fn check_ollama_health(&self) -> bool {
        match self
            .client
            .get(format!("{}/api/tags", self.config.ollama_endpoint))
            .send()
            .await
        {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }
}

/// Service statistics
#[derive(Debug, Clone, Serialize)]
pub struct ReasoningStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_hit_rate: f64,
}

// ============================================================================
// Benchmark Service
// ============================================================================

/// Standard benchmark questions for performance consensus testing
pub struct BenchmarkSuite {
    pub questions: Vec<BenchmarkQuestion>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkQuestion {
    pub id: String,
    pub question: String,
    pub expected_keywords: Vec<String>,
    pub category: String,
}

impl BenchmarkSuite {
    pub fn default_suite() -> Self {
        Self {
            questions: vec![
                BenchmarkQuestion {
                    id: "math_1".to_string(),
                    question: "What is 15% of 240?".to_string(),
                    expected_keywords: vec!["36".to_string()],
                    category: "math".to_string(),
                },
                BenchmarkQuestion {
                    id: "logic_1".to_string(),
                    question: "If all roses are flowers and some flowers fade quickly, can we conclude that some roses fade quickly?".to_string(),
                    expected_keywords: vec!["no".to_string(), "cannot".to_string(), "not necessarily".to_string()],
                    category: "logic".to_string(),
                },
                BenchmarkQuestion {
                    id: "fact_1".to_string(),
                    question: "What is the capital of France?".to_string(),
                    expected_keywords: vec!["Paris".to_string()],
                    category: "factual".to_string(),
                },
                BenchmarkQuestion {
                    id: "code_1".to_string(),
                    question: "Write a Python function to check if a number is prime.".to_string(),
                    expected_keywords: vec!["def".to_string(), "prime".to_string(), "return".to_string()],
                    category: "coding".to_string(),
                },
            ],
        }
    }

    /// Score a response against expected keywords
    pub fn score_response(&self, question_id: &str, response: &str) -> f64 {
        let question = self.questions.iter().find(|q| q.id == question_id);

        match question {
            Some(q) => {
                let response_lower = response.to_lowercase();
                let matches = q
                    .expected_keywords
                    .iter()
                    .filter(|kw| response_lower.contains(&kw.to_lowercase()))
                    .count();

                if q.expected_keywords.is_empty() {
                    0.5 // No keywords to match
                } else {
                    matches as f64 / q.expected_keywords.len() as f64
                }
            }
            None => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_tier_thresholds() {
        assert!(ModelTier::Local7B.complexity_threshold() < ModelTier::Local13B.complexity_threshold());
        assert!(ModelTier::Local13B.complexity_threshold() < ModelTier::Local70B.complexity_threshold());
    }

    #[tokio::test]
    async fn test_router_basic_routing() {
        let router = ModelRouter::new();

        let request = ReasoningRequest {
            task_id: 1,
            intent: "What is 2+2?".to_string(),
            context: None,
            complexity_score: 1000,
            workflow: WorkflowClass::ExpressLocal,
            max_tokens: None,
            temperature: None,
            force_fresh: false,
            metadata: HashMap::new(),
        };

        let decision = router.route(&request).await;
        assert_eq!(decision.tier, ModelTier::Local7B);
    }

    #[tokio::test]
    async fn test_router_high_complexity() {
        let router = ModelRouter::new();

        let request = ReasoningRequest {
            task_id: 1,
            intent: "Complex multi-step reasoning task".to_string(),
            context: None,
            complexity_score: 9000,
            workflow: WorkflowClass::HighPrecision,
            max_tokens: None,
            temperature: None,
            force_fresh: false,
            metadata: HashMap::new(),
        };

        let decision = router.route(&request).await;
        assert_eq!(decision.tier, ModelTier::CloudAPI);
    }

    #[test]
    fn test_benchmark_scoring() {
        let suite = BenchmarkSuite::default_suite();

        // Correct answer
        let score1 = suite.score_response("math_1", "The answer is 36");
        assert!(score1 > 0.9);

        // Wrong answer
        let score2 = suite.score_response("math_1", "The answer is 42");
        assert!(score2 < 0.1);
    }
}

