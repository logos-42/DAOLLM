//! TRO Knowledge Graph Service
//!
//! Implements the verification layer's knowledge base:
//! - Triplet extraction from LLM outputs
//! - Graph storage and querying (Neo4j compatible)
//! - Fact verification against stored knowledge
//! - Merkle root computation for on-chain synchronization

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Types
// ============================================================================

/// A knowledge triplet (subject, predicate, object)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Triplet {
    /// Subject entity
    pub subject: String,
    /// Predicate/relationship
    pub predicate: String,
    /// Object entity
    pub object: String,
    /// Confidence score (0-10000 bps)
    pub confidence: u16,
    /// Source of this triplet
    pub source: TripletSource,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last verification timestamp
    pub verified_at: Option<DateTime<Utc>>,
}

impl Triplet {
    pub fn new(subject: &str, predicate: &str, object: &str, confidence: u16, source: TripletSource) -> Self {
        Self {
            subject: subject.to_string(),
            predicate: predicate.to_string(),
            object: object.to_string(),
            confidence,
            source,
            created_at: Utc::now(),
            verified_at: None,
        }
    }

    /// Compute hash of this triplet
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.subject.as_bytes());
        hasher.update(self.predicate.as_bytes());
        hasher.update(self.object.as_bytes());
        hasher.finalize().into()
    }

    /// Canonical string representation
    pub fn canonical(&self) -> String {
        format!(
            "({}, {}, {})",
            self.subject.to_lowercase().trim(),
            self.predicate.to_lowercase().trim(),
            self.object.to_lowercase().trim()
        )
    }
}

/// Source of a triplet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TripletSource {
    /// Extracted from LLM output
    LLMExtraction,
    /// Imported from external knowledge base
    ExternalImport,
    /// Human-verified fact
    HumanVerified,
    /// Derived from other triplets
    Derived,
}

/// An entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub label: String,
    pub entity_type: String,
    pub properties: HashMap<String, String>,
    pub aliases: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// A relation type in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationType {
    pub name: String,
    pub inverse: Option<String>,
    pub transitive: bool,
    pub symmetric: bool,
}

/// Fact verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactVerification {
    pub claim: Triplet,
    pub supported: bool,
    pub confidence: f64,
    pub supporting_triplets: Vec<Triplet>,
    pub contradicting_triplets: Vec<Triplet>,
    pub verification_path: Vec<String>,
}

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct KnowledgeGraphConfig {
    /// Neo4j connection URL (optional)
    pub neo4j_url: Option<String>,
    /// Neo4j username
    pub neo4j_user: Option<String>,
    /// Neo4j password
    pub neo4j_password: Option<String>,
    /// Enable in-memory graph for testing
    pub enable_memory_graph: bool,
    /// Maximum entities to keep in memory
    pub max_memory_entities: usize,
    /// Confidence threshold for accepting facts
    pub acceptance_threshold: u16,
}

impl Default for KnowledgeGraphConfig {
    fn default() -> Self {
        Self {
            neo4j_url: None,
            neo4j_user: None,
            neo4j_password: None,
            enable_memory_graph: true,
            max_memory_entities: 100000,
            acceptance_threshold: 7000, // 70%
        }
    }
}

// ============================================================================
// Knowledge Graph Service
// ============================================================================

pub struct KnowledgeGraphService {
    config: KnowledgeGraphConfig,
    /// In-memory entity storage
    entities: RwLock<HashMap<String, Entity>>,
    /// In-memory triplet storage
    triplets: RwLock<HashSet<Triplet>>,
    /// Entity index by label
    label_index: RwLock<HashMap<String, String>>,
    /// Relation types
    relation_types: RwLock<HashMap<String, RelationType>>,
    /// Statistics
    stats: RwLock<KGStats>,
}

#[derive(Debug, Clone, Default)]
pub struct KGStats {
    pub entity_count: u64,
    pub triplet_count: u64,
    pub relation_type_count: u64,
    pub verifications_performed: u64,
    pub verifications_passed: u64,
}

impl KnowledgeGraphService {
    pub async fn new(config: KnowledgeGraphConfig) -> Result<Self> {
        let service = Self {
            config,
            entities: RwLock::new(HashMap::new()),
            triplets: RwLock::new(HashSet::new()),
            label_index: RwLock::new(HashMap::new()),
            relation_types: RwLock::new(HashMap::new()),
            stats: RwLock::new(KGStats::default()),
        };

        // Initialize common relation types
        service.init_relation_types().await;

        Ok(service)
    }

    /// Initialize common relation types
    async fn init_relation_types(&self) {
        let mut types = self.relation_types.write().await;

        types.insert(
            "is_a".to_string(),
            RelationType {
                name: "is_a".to_string(),
                inverse: None,
                transitive: true,
                symmetric: false,
            },
        );

        types.insert(
            "part_of".to_string(),
            RelationType {
                name: "part_of".to_string(),
                inverse: Some("has_part".to_string()),
                transitive: true,
                symmetric: false,
            },
        );

        types.insert(
            "located_in".to_string(),
            RelationType {
                name: "located_in".to_string(),
                inverse: Some("contains".to_string()),
                transitive: true,
                symmetric: false,
            },
        );

        types.insert(
            "equal_to".to_string(),
            RelationType {
                name: "equal_to".to_string(),
                inverse: Some("equal_to".to_string()),
                transitive: true,
                symmetric: true,
            },
        );
    }

    /// Add an entity to the graph
    pub async fn add_entity(&self, entity: Entity) -> Result<()> {
        let mut entities = self.entities.write().await;
        let mut label_index = self.label_index.write().await;

        // Check capacity
        if entities.len() >= self.config.max_memory_entities {
            return Err(anyhow!("Entity capacity exceeded"));
        }

        // Index by label and aliases
        label_index.insert(entity.label.to_lowercase(), entity.id.clone());
        for alias in &entity.aliases {
            label_index.insert(alias.to_lowercase(), entity.id.clone());
        }

        entities.insert(entity.id.clone(), entity);

        let mut stats = self.stats.write().await;
        stats.entity_count += 1;

        Ok(())
    }

    /// Add a triplet to the graph
    pub async fn add_triplet(&self, triplet: Triplet) -> Result<()> {
        // Ensure entities exist
        let entities = self.entities.read().await;
        let label_index = self.label_index.read().await;

        // Auto-create entities if they don't exist
        drop(entities);
        drop(label_index);

        if !self.entity_exists(&triplet.subject).await {
            self.add_entity(Entity {
                id: format!("auto_{}", triplet.subject.to_lowercase().replace(' ', "_")),
                label: triplet.subject.clone(),
                entity_type: "auto".to_string(),
                properties: HashMap::new(),
                aliases: vec![],
                created_at: Utc::now(),
            })
            .await?;
        }

        if !self.entity_exists(&triplet.object).await {
            self.add_entity(Entity {
                id: format!("auto_{}", triplet.object.to_lowercase().replace(' ', "_")),
                label: triplet.object.clone(),
                entity_type: "auto".to_string(),
                properties: HashMap::new(),
                aliases: vec![],
                created_at: Utc::now(),
            })
            .await?;
        }

        let mut triplets = self.triplets.write().await;
        triplets.insert(triplet);

        let mut stats = self.stats.write().await;
        stats.triplet_count += 1;

        Ok(())
    }

    /// Check if entity exists
    async fn entity_exists(&self, label: &str) -> bool {
        let label_index = self.label_index.read().await;
        label_index.contains_key(&label.to_lowercase())
    }

    /// Extract triplets from text
    pub async fn extract_triplets(&self, text: &str) -> Vec<Triplet> {
        let mut triplets = Vec::new();

        // Simple rule-based extraction (for MVP)
        // In production, use NLP models

        let sentences: Vec<&str> = text.split('.').collect();

        for sentence in sentences {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }

            // Pattern: "X is Y" -> (X, is_a, Y)
            if let Some(triplet) = self.extract_is_pattern(sentence) {
                triplets.push(triplet);
            }

            // Pattern: "X located in Y" -> (X, located_in, Y)
            if let Some(triplet) = self.extract_location_pattern(sentence) {
                triplets.push(triplet);
            }

            // Pattern: "X has Y" -> (X, has, Y)
            if let Some(triplet) = self.extract_has_pattern(sentence) {
                triplets.push(triplet);
            }

            // Pattern: "X = Y" or "X equals Y"
            if let Some(triplet) = self.extract_equals_pattern(sentence) {
                triplets.push(triplet);
            }
        }

        triplets
    }

    fn extract_is_pattern(&self, sentence: &str) -> Option<Triplet> {
        let patterns = [" is ", " is a ", " is an ", " are "];

        for pattern in patterns {
            if let Some(idx) = sentence.to_lowercase().find(pattern) {
                let subject = sentence[..idx].trim();
                let object = sentence[idx + pattern.len()..].trim();

                if !subject.is_empty() && !object.is_empty() {
                    return Some(Triplet::new(
                        subject,
                        "is_a",
                        object,
                        7000,
                        TripletSource::LLMExtraction,
                    ));
                }
            }
        }

        None
    }

    fn extract_location_pattern(&self, sentence: &str) -> Option<Triplet> {
        let patterns = [
            " located in ",
            " is in ",
            " in ",
            " capital of ",
        ];

        for pattern in patterns {
            if let Some(idx) = sentence.to_lowercase().find(pattern) {
                let subject = sentence[..idx].trim();
                let object = sentence[idx + pattern.len()..].trim();

                if !subject.is_empty() && !object.is_empty() {
                    return Some(Triplet::new(
                        subject,
                        "located_in",
                        object,
                        6500,
                        TripletSource::LLMExtraction,
                    ));
                }
            }
        }

        None
    }

    fn extract_has_pattern(&self, sentence: &str) -> Option<Triplet> {
        let patterns = [" has ", " have ", " contains "];

        for pattern in patterns {
            if let Some(idx) = sentence.to_lowercase().find(pattern) {
                let subject = sentence[..idx].trim();
                let object = sentence[idx + pattern.len()..].trim();

                if !subject.is_empty() && !object.is_empty() {
                    return Some(Triplet::new(
                        subject,
                        "has",
                        object,
                        6000,
                        TripletSource::LLMExtraction,
                    ));
                }
            }
        }

        None
    }

    fn extract_equals_pattern(&self, sentence: &str) -> Option<Triplet> {
        // Check for "X = Y" pattern
        if let Some(idx) = sentence.find('=') {
            let subject = sentence[..idx].trim();
            let object = sentence[idx + 1..].trim();

            if !subject.is_empty() && !object.is_empty() {
                return Some(Triplet::new(
                    subject,
                    "equal_to",
                    object,
                    9000,
                    TripletSource::LLMExtraction,
                ));
            }
        }

        // Check for "X equals Y" pattern
        if let Some(idx) = sentence.to_lowercase().find(" equals ") {
            let subject = sentence[..idx].trim();
            let object = sentence[idx + 8..].trim();

            if !subject.is_empty() && !object.is_empty() {
                return Some(Triplet::new(
                    subject,
                    "equal_to",
                    object,
                    9000,
                    TripletSource::LLMExtraction,
                ));
            }
        }

        None
    }

    /// Verify a claim against the knowledge graph
    pub async fn verify_fact(&self, claim: &Triplet) -> FactVerification {
        let triplets = self.triplets.read().await;
        let mut stats = self.stats.write().await;
        stats.verifications_performed += 1;

        let mut supporting = Vec::new();
        let mut contradicting = Vec::new();
        let mut verification_path = Vec::new();

        let claim_canonical = claim.canonical();

        // Direct match
        for stored in triplets.iter() {
            if stored.canonical() == claim_canonical {
                supporting.push(stored.clone());
                verification_path.push(format!("Direct match: {}", stored.canonical()));
            }
        }

        // Check for contradictions
        for stored in triplets.iter() {
            // Same subject and predicate but different object
            if stored.subject.to_lowercase() == claim.subject.to_lowercase()
                && stored.predicate.to_lowercase() == claim.predicate.to_lowercase()
                && stored.object.to_lowercase() != claim.object.to_lowercase()
            {
                // For non-multi-valued predicates, this is a contradiction
                if !self.is_multi_valued(&stored.predicate) {
                    contradicting.push(stored.clone());
                    verification_path.push(format!("Contradiction: {}", stored.canonical()));
                }
            }
        }

        // Check transitive relations
        if supporting.is_empty() {
            if let Some(path) = self.find_transitive_path(claim, &triplets).await {
                verification_path.extend(path.iter().map(|t| format!("Transitive: {}", t.canonical())));
                supporting.extend(path);
            }
        }

        // Calculate confidence
        let confidence = if !supporting.is_empty() && contradicting.is_empty() {
            let avg_confidence: f64 = supporting.iter().map(|t| t.confidence as f64).sum::<f64>()
                / supporting.len() as f64;
            avg_confidence / 10000.0
        } else if !contradicting.is_empty() {
            0.0
        } else {
            0.5 // Unknown
        };

        let supported = confidence >= (self.config.acceptance_threshold as f64 / 10000.0);
        if supported {
            stats.verifications_passed += 1;
        }

        FactVerification {
            claim: claim.clone(),
            supported,
            confidence,
            supporting_triplets: supporting,
            contradicting_triplets: contradicting,
            verification_path,
        }
    }

    fn is_multi_valued(&self, predicate: &str) -> bool {
        matches!(
            predicate.to_lowercase().as_str(),
            "has" | "contains" | "part_of" | "related_to"
        )
    }

    async fn find_transitive_path(&self, claim: &Triplet, triplets: &HashSet<Triplet>) -> Option<Vec<Triplet>> {
        // Simple 2-hop transitive search
        let subject_lower = claim.subject.to_lowercase();
        let object_lower = claim.object.to_lowercase();
        let predicate_lower = claim.predicate.to_lowercase();

        // Check if predicate is transitive
        let relation_types = self.relation_types.read().await;
        let is_transitive = relation_types
            .get(&predicate_lower)
            .map(|r| r.transitive)
            .unwrap_or(false);

        if !is_transitive {
            return None;
        }

        // Find intermediate nodes
        for t1 in triplets.iter() {
            if t1.subject.to_lowercase() == subject_lower
                && t1.predicate.to_lowercase() == predicate_lower
            {
                for t2 in triplets.iter() {
                    if t2.subject.to_lowercase() == t1.object.to_lowercase()
                        && t2.predicate.to_lowercase() == predicate_lower
                        && t2.object.to_lowercase() == object_lower
                    {
                        return Some(vec![t1.clone(), t2.clone()]);
                    }
                }
            }
        }

        None
    }

    /// Compute Merkle root of all triplets
    pub async fn compute_merkle_root(&self) -> [u8; 32] {
        let triplets = self.triplets.read().await;

        if triplets.is_empty() {
            return [0u8; 32];
        }

        // Collect all triplet hashes
        let mut hashes: Vec<[u8; 32]> = triplets.iter().map(|t| t.hash()).collect();

        // Sort for deterministic order
        hashes.sort();

        // Build Merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate if odd
                }
                next_level.push(hasher.finalize().into());
            }

            hashes = next_level;
        }

        hashes[0]
    }

    /// Get graph statistics
    pub async fn get_stats(&self) -> KGStats {
        self.stats.read().await.clone()
    }

    /// Query triplets by subject
    pub async fn query_by_subject(&self, subject: &str) -> Vec<Triplet> {
        let triplets = self.triplets.read().await;
        let subject_lower = subject.to_lowercase();

        triplets
            .iter()
            .filter(|t| t.subject.to_lowercase() == subject_lower)
            .cloned()
            .collect()
    }

    /// Query triplets by object
    pub async fn query_by_object(&self, object: &str) -> Vec<Triplet> {
        let triplets = self.triplets.read().await;
        let object_lower = object.to_lowercase();

        triplets
            .iter()
            .filter(|t| t.object.to_lowercase() == object_lower)
            .cloned()
            .collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_triplet_extraction() {
        let service = KnowledgeGraphService::new(KnowledgeGraphConfig::default())
            .await
            .unwrap();

        let text = "Paris is the capital of France. The Eiffel Tower is located in Paris.";
        let triplets = service.extract_triplets(text).await;

        assert!(!triplets.is_empty());
    }

    #[tokio::test]
    async fn test_add_and_verify() {
        let service = KnowledgeGraphService::new(KnowledgeGraphConfig::default())
            .await
            .unwrap();

        // Add a known fact
        let triplet = Triplet::new(
            "Paris",
            "is_a",
            "city",
            9000,
            TripletSource::HumanVerified,
        );
        service.add_triplet(triplet.clone()).await.unwrap();

        // Verify the same fact
        let verification = service.verify_fact(&triplet).await;
        assert!(verification.supported);
        assert!(verification.confidence > 0.8);
    }

    #[tokio::test]
    async fn test_merkle_root() {
        let service = KnowledgeGraphService::new(KnowledgeGraphConfig::default())
            .await
            .unwrap();

        // Add triplets
        service
            .add_triplet(Triplet::new("A", "is", "B", 9000, TripletSource::HumanVerified))
            .await
            .unwrap();
        service
            .add_triplet(Triplet::new("C", "is", "D", 9000, TripletSource::HumanVerified))
            .await
            .unwrap();

        let root = service.compute_merkle_root().await;
        assert_ne!(root, [0u8; 32]);
    }

    #[tokio::test]
    async fn test_transitive_verification() {
        let service = KnowledgeGraphService::new(KnowledgeGraphConfig::default())
            .await
            .unwrap();

        // Paris is in France, France is in Europe
        service
            .add_triplet(Triplet::new(
                "Paris",
                "located_in",
                "France",
                9000,
                TripletSource::HumanVerified,
            ))
            .await
            .unwrap();
        service
            .add_triplet(Triplet::new(
                "France",
                "located_in",
                "Europe",
                9000,
                TripletSource::HumanVerified,
            ))
            .await
            .unwrap();

        // Verify transitive: Paris is in Europe
        let claim = Triplet::new("Paris", "located_in", "Europe", 8000, TripletSource::LLMExtraction);
        let verification = service.verify_fact(&claim).await;

        assert!(verification.supported);
        assert!(!verification.verification_path.is_empty());
    }
}

