//! TRO Multi-Perspective Verification Service
//!
//! Implements the verification layer of TRO pipeline:
//! - NLI-based fact consistency checking
//! - Cross-model validation (LLM審LLM)
//! - Hallucination detection
//! - Score aggregation with reputation weighting

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::knowledge_graph_service::{KnowledgeGraphService, Triplet, TripletSource};
use super::reasoning_service::{ModelTier, ReasoningRequest, ReasoningResponse, ReasoningService, WorkflowClass};

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct VerificationConfig {
    /// Minimum verification score to pass (0-10000 bps)
    pub min_pass_score: u16,
    /// Weight for semantic similarity in final score
    pub semantic_weight: f64,
    /// Weight for fact consistency in final score
    pub fact_weight: f64,
    /// Weight for KG matching in final score
    pub kg_weight: f64,
    /// Number of cross-validation models to use
    pub cross_validation_count: usize,
    /// Enable hallucination detection
    pub enable_hallucination_detection: bool,
    /// Hallucination keywords to flag
    pub hallucination_indicators: Vec<String>,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            min_pass_score: 7000, // 70%
            semantic_weight: 0.3,
            fact_weight: 0.4,
            kg_weight: 0.3,
            cross_validation_count: 2,
            enable_hallucination_detection: true,
            hallucination_indicators: vec![
                "I don't have access".to_string(),
                "As an AI".to_string(),
                "I cannot verify".to_string(),
                "I'm not sure".to_string(),
                "might be".to_string(),
                "possibly".to_string(),
                "I think".to_string(),
                "In my opinion".to_string(),
            ],
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Verification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub task_id: u64,
    pub original_query: String,
    pub reasoning_response: String,
    pub model_used: String,
    pub claimed_confidence: u16,
    pub metadata: HashMap<String, String>,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub task_id: u64,
    pub passed: bool,
    pub final_score_bps: u16,
    pub semantic_score: f64,
    pub fact_consistency_score: f64,
    pub kg_match_score: f64,
    pub hallucination_detected: bool,
    pub hallucination_reasons: Vec<String>,
    pub cross_validation_results: Vec<CrossValidationResult>,
    pub extracted_triplets: Vec<Triplet>,
    pub verification_time_ms: u64,
    pub timestamp: DateTime<Utc>,
}

/// Cross-validation result from another model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossValidationResult {
    pub validator_model: String,
    pub agrees: bool,
    pub confidence: f64,
    pub discrepancies: Vec<String>,
}

/// NLI classification result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NLILabel {
    Entailment,
    Contradiction,
    Neutral,
}

// ============================================================================
// Verification Service
// ============================================================================

pub struct VerificationService {
    config: VerificationConfig,
    kg_service: Arc<KnowledgeGraphService>,
    reasoning_service: Option<Arc<ReasoningService>>,
    /// Cache of recent verifications
    verification_cache: RwLock<HashMap<u64, VerificationResult>>,
    /// Statistics
    stats: RwLock<VerificationStats>,
}

#[derive(Debug, Clone, Default)]
pub struct VerificationStats {
    pub total_verifications: u64,
    pub passed: u64,
    pub failed: u64,
    pub hallucinations_detected: u64,
    pub avg_score: f64,
}

impl VerificationService {
    pub fn new(
        config: VerificationConfig,
        kg_service: Arc<KnowledgeGraphService>,
        reasoning_service: Option<Arc<ReasoningService>>,
    ) -> Self {
        Self {
            config,
            kg_service,
            reasoning_service,
            verification_cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(VerificationStats::default()),
        }
    }

    /// Verify a reasoning response
    pub async fn verify(&self, request: VerificationRequest) -> Result<VerificationResult> {
        let start = std::time::Instant::now();
        info!("Starting verification for task {}", request.task_id);

        // Step 1: Extract triplets from response
        let extracted_triplets = self
            .kg_service
            .extract_triplets(&request.reasoning_response)
            .await;
        debug!(
            "Extracted {} triplets from response",
            extracted_triplets.len()
        );

        // Step 2: Check fact consistency against KG
        let kg_match_score = self.compute_kg_score(&extracted_triplets).await;

        // Step 3: Semantic similarity check (simplified)
        let semantic_score = self.compute_semantic_score(&request).await;

        // Step 4: Cross-validation with other models
        let cross_results = self.cross_validate(&request).await;
        let fact_consistency_score = self.compute_fact_consistency(&cross_results);

        // Step 5: Hallucination detection
        let (hallucination_detected, hallucination_reasons) =
            self.detect_hallucinations(&request.reasoning_response);

        // Step 6: Compute final score
        let weighted_score = (semantic_score * self.config.semantic_weight
            + fact_consistency_score * self.config.fact_weight
            + kg_match_score * self.config.kg_weight)
            / (self.config.semantic_weight + self.config.fact_weight + self.config.kg_weight);

        // Apply hallucination penalty
        let final_score = if hallucination_detected {
            weighted_score * 0.5
        } else {
            weighted_score
        };

        let final_score_bps = (final_score * 10000.0) as u16;
        let passed = final_score_bps >= self.config.min_pass_score && !hallucination_detected;

        // Store triplets in KG if verification passed
        if passed {
            for triplet in &extracted_triplets {
                let _ = self.kg_service.add_triplet(triplet.clone()).await;
            }
        }

        let result = VerificationResult {
            task_id: request.task_id,
            passed,
            final_score_bps,
            semantic_score,
            fact_consistency_score,
            kg_match_score,
            hallucination_detected,
            hallucination_reasons,
            cross_validation_results: cross_results,
            extracted_triplets,
            verification_time_ms: start.elapsed().as_millis() as u64,
            timestamp: Utc::now(),
        };

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_verifications += 1;
            if passed {
                stats.passed += 1;
            } else {
                stats.failed += 1;
            }
            if hallucination_detected {
                stats.hallucinations_detected += 1;
            }
            // Running average
            stats.avg_score = ((stats.avg_score * (stats.total_verifications - 1) as f64)
                + final_score)
                / stats.total_verifications as f64;
        }

        // Cache result
        {
            let mut cache = self.verification_cache.write().await;
            cache.insert(request.task_id, result.clone());
        }

        info!(
            "Verification complete for task {}: passed={}, score={}",
            request.task_id, passed, final_score_bps
        );

        Ok(result)
    }

    /// Compute KG matching score
    async fn compute_kg_score(&self, triplets: &[Triplet]) -> f64 {
        if triplets.is_empty() {
            return 0.5; // Neutral if no triplets
        }

        let mut total_confidence = 0.0;
        let mut verified_count = 0;

        for triplet in triplets {
            let verification = self.kg_service.verify_fact(triplet).await;
            if verification.supported {
                total_confidence += verification.confidence;
                verified_count += 1;
            }
        }

        if verified_count == 0 {
            0.5 // Unknown
        } else {
            total_confidence / verified_count as f64
        }
    }

    /// Compute semantic similarity score (simplified)
    async fn compute_semantic_score(&self, request: &VerificationRequest) -> f64 {
        // Simple heuristics for MVP
        // In production, use SBERT embeddings

        let response_lower = request.reasoning_response.to_lowercase();
        let query_lower = request.original_query.to_lowercase();

        // Check if response addresses the query
        let query_words: std::collections::HashSet<_> = query_lower
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        let response_words: std::collections::HashSet<_> = response_lower
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .collect();

        if query_words.is_empty() {
            return 0.5;
        }

        let overlap = query_words.intersection(&response_words).count();
        let query_coverage = overlap as f64 / query_words.len() as f64;

        // Response length heuristic (very short responses are suspicious)
        let length_score = if request.reasoning_response.len() < 20 {
            0.3
        } else if request.reasoning_response.len() < 100 {
            0.7
        } else {
            1.0
        };

        (query_coverage * 0.7 + length_score * 0.3).min(1.0)
    }

    /// Cross-validate with other models
    async fn cross_validate(&self, request: &VerificationRequest) -> Vec<CrossValidationResult> {
        let mut results = Vec::new();

        // If no reasoning service, skip cross-validation
        let reasoning = match &self.reasoning_service {
            Some(r) => r,
            None => return results,
        };

        // Create verification prompt
        let verification_prompt = format!(
            "Verify the following response to the query.\n\n\
             Query: {}\n\n\
             Response to verify: {}\n\n\
             Is this response accurate and complete? \
             Respond with YES or NO, followed by a brief explanation.",
            request.original_query, request.reasoning_response
        );

        // Use a different model tier for cross-validation
        let validation_request = ReasoningRequest {
            task_id: request.task_id * 1000, // Different ID
            intent: verification_prompt,
            context: None,
            complexity_score: 3000, // Simple validation task
            workflow: WorkflowClass::Standard,
            max_tokens: Some(256),
            temperature: Some(0.1),
            force_fresh: true,
            metadata: HashMap::new(),
        };

        match reasoning.process(validation_request).await {
            Ok(response) => {
                let response_lower = response.result.to_lowercase();
                let agrees = response_lower.contains("yes")
                    || response_lower.contains("correct")
                    || response_lower.contains("accurate");

                let mut discrepancies = Vec::new();
                if response_lower.contains("no")
                    || response_lower.contains("incorrect")
                    || response_lower.contains("inaccurate")
                {
                    discrepancies.push(response.result.clone());
                }

                results.push(CrossValidationResult {
                    validator_model: response.model_used,
                    agrees,
                    confidence: response.confidence_bps as f64 / 10000.0,
                    discrepancies,
                });
            }
            Err(e) => {
                warn!("Cross-validation failed: {}", e);
            }
        }

        results
    }

    /// Compute fact consistency from cross-validation
    fn compute_fact_consistency(&self, results: &[CrossValidationResult]) -> f64 {
        if results.is_empty() {
            return 0.7; // Default moderate confidence
        }

        let agreement_count = results.iter().filter(|r| r.agrees).count();
        let weighted_agreement: f64 = results
            .iter()
            .filter(|r| r.agrees)
            .map(|r| r.confidence)
            .sum();

        let total_weight: f64 = results.iter().map(|r| r.confidence).sum();

        if total_weight == 0.0 {
            return 0.5;
        }

        weighted_agreement / total_weight
    }

    /// Detect hallucinations in response
    fn detect_hallucinations(&self, response: &str) -> (bool, Vec<String>) {
        if !self.config.enable_hallucination_detection {
            return (false, vec![]);
        }

        let response_lower = response.to_lowercase();
        let mut reasons = Vec::new();

        for indicator in &self.config.hallucination_indicators {
            if response_lower.contains(&indicator.to_lowercase()) {
                reasons.push(format!("Contains uncertain phrase: '{}'", indicator));
            }
        }

        // Check for self-contradictions (simplified)
        if response_lower.contains("but actually") || response_lower.contains("however, that's wrong")
        {
            reasons.push("Contains self-contradiction".to_string());
        }

        // Check for impossible claims
        let impossible_patterns = [
            "100% certain",
            "absolutely guaranteed",
            "impossible to fail",
            "never wrong",
        ];
        for pattern in impossible_patterns {
            if response_lower.contains(pattern) {
                reasons.push(format!("Contains overconfident claim: '{}'", pattern));
            }
        }

        (!reasons.is_empty(), reasons)
    }

    /// Get verification statistics
    pub async fn get_stats(&self) -> VerificationStats {
        self.stats.read().await.clone()
    }

    /// Get cached verification result
    pub async fn get_cached(&self, task_id: u64) -> Option<VerificationResult> {
        let cache = self.verification_cache.read().await;
        cache.get(&task_id).cloned()
    }
}

// ============================================================================
// Truth Discovery Algorithm (SenteTruth-inspired)
// ============================================================================

/// Truth discovery for aggregating multiple responses
pub struct TruthDiscovery {
    /// Node reputation weights
    reputation_weights: HashMap<String, f64>,
    /// Semantic similarity threshold
    similarity_threshold: f64,
}

impl TruthDiscovery {
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            reputation_weights: HashMap::new(),
            similarity_threshold,
        }
    }

    /// Set reputation weight for a node
    pub fn set_reputation(&mut self, node_id: &str, weight: f64) {
        self.reputation_weights.insert(node_id.to_string(), weight);
    }

    /// Aggregate multiple responses using truth discovery
    pub fn aggregate(&self, responses: Vec<(String, String, f64)>) -> Option<AggregatedResult> {
        // responses: Vec<(node_id, response_text, claimed_confidence)>

        if responses.is_empty() {
            return None;
        }

        if responses.len() == 1 {
            let (node_id, text, confidence) = &responses[0];
            return Some(AggregatedResult {
                consensus_text: text.clone(),
                consensus_confidence: *confidence,
                contributing_nodes: vec![node_id.clone()],
                agreement_ratio: 1.0,
            });
        }

        // Group similar responses
        let clusters = self.cluster_responses(&responses);

        // Find largest cluster
        let largest_cluster = clusters
            .iter()
            .max_by_key(|c| c.len())
            .cloned()
            .unwrap_or_default();

        if largest_cluster.is_empty() {
            return None;
        }

        // Compute weighted consensus
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        let mut contributing_nodes = Vec::new();

        for idx in &largest_cluster {
            let (node_id, _, confidence) = &responses[*idx];
            let reputation = self.reputation_weights.get(node_id).copied().unwrap_or(1.0);
            let weight = reputation * confidence;

            weighted_sum += weight;
            total_weight += reputation;
            contributing_nodes.push(node_id.clone());
        }

        let consensus_confidence = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.5
        };

        // Use most common response in cluster
        let consensus_text = responses[largest_cluster[0]].1.clone();

        let agreement_ratio = largest_cluster.len() as f64 / responses.len() as f64;

        Some(AggregatedResult {
            consensus_text,
            consensus_confidence,
            contributing_nodes,
            agreement_ratio,
        })
    }

    /// Cluster responses by similarity
    fn cluster_responses(&self, responses: &[(String, String, f64)]) -> Vec<Vec<usize>> {
        let mut clusters: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; responses.len()];

        for i in 0..responses.len() {
            if assigned[i] {
                continue;
            }

            let mut cluster = vec![i];
            assigned[i] = true;

            for j in (i + 1)..responses.len() {
                if assigned[j] {
                    continue;
                }

                let similarity = self.compute_jaccard(&responses[i].1, &responses[j].1);
                if similarity >= self.similarity_threshold {
                    cluster.push(j);
                    assigned[j] = true;
                }
            }

            clusters.push(cluster);
        }

        clusters
    }

    /// Compute Jaccard similarity between two texts
    fn compute_jaccard(&self, text1: &str, text2: &str) -> f64 {
        let words1: std::collections::HashSet<_> = text1.to_lowercase().split_whitespace().collect();
        let words2: std::collections::HashSet<_> = text2.to_lowercase().split_whitespace().collect();

        if words1.is_empty() && words2.is_empty() {
            return 1.0;
        }

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// Result of truth discovery aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult {
    pub consensus_text: String,
    pub consensus_confidence: f64,
    pub contributing_nodes: Vec<String>,
    pub agreement_ratio: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hallucination_detection() {
        let config = VerificationConfig::default();
        let kg_service = Arc::new(futures::executor::block_on(async {
            KnowledgeGraphService::new(super::super::knowledge_graph_service::KnowledgeGraphConfig::default())
                .await
                .unwrap()
        }));
        let service = VerificationService::new(config, kg_service, None);

        let (detected, reasons) =
            service.detect_hallucinations("I'm not sure about this, but I think it might be correct.");
        assert!(detected);
        assert!(!reasons.is_empty());

        let (detected, _) = service.detect_hallucinations("Paris is the capital of France.");
        assert!(!detected);
    }

    #[test]
    fn test_truth_discovery_aggregation() {
        let mut td = TruthDiscovery::new(0.5);
        td.set_reputation("node1", 1.0);
        td.set_reputation("node2", 0.8);
        td.set_reputation("node3", 0.6);

        let responses = vec![
            ("node1".to_string(), "The answer is 42".to_string(), 0.9),
            ("node2".to_string(), "The answer is 42".to_string(), 0.85),
            ("node3".to_string(), "The answer is 43".to_string(), 0.7),
        ];

        let result = td.aggregate(responses).unwrap();
        assert_eq!(result.contributing_nodes.len(), 2);
        assert!(result.agreement_ratio > 0.5);
    }

    #[test]
    fn test_jaccard_similarity() {
        let td = TruthDiscovery::new(0.5);

        let sim1 = td.compute_jaccard("hello world", "hello world");
        assert!((sim1 - 1.0).abs() < 0.01);

        let sim2 = td.compute_jaccard("hello world", "goodbye universe");
        assert!(sim2 < 0.5);

        let sim3 = td.compute_jaccard("the quick brown fox", "a quick brown dog");
        assert!(sim3 > 0.3 && sim3 < 0.8);
    }
}

