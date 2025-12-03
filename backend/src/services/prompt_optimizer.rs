//! TRO Prompt Optimizer Service
//!
//! Implements prompt compression and optimization:
//! - Context compression (remove redundancy, keep key entities)
//! - Knowledge graph reference substitution
//! - Structured I/O via JSON Schema
//! - Token estimation and budget management

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

// ============================================================================
// Configuration
// ============================================================================

/// Prompt optimizer configuration
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Maximum input tokens (approximate)
    pub max_input_tokens: usize,
    /// Target compression ratio (0.0 - 1.0)
    pub target_compression_ratio: f64,
    /// Enable knowledge graph reference substitution
    pub enable_kg_substitution: bool,
    /// Enable structured output via JSON schema
    pub enable_json_schema: bool,
    /// Stop words to remove during compression
    pub stop_words: HashSet<String>,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        let stop_words: HashSet<String> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "dare",
            "ought", "used", "to", "of", "in", "for", "on", "with", "at", "by",
            "from", "as", "into", "through", "during", "before", "after",
            "above", "below", "between", "under", "again", "further", "then",
            "once", "here", "there", "when", "where", "why", "how", "all",
            "each", "few", "more", "most", "other", "some", "such", "no", "nor",
            "not", "only", "own", "same", "so", "than", "too", "very", "just",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            max_input_tokens: 4096,
            target_compression_ratio: 0.6,
            enable_kg_substitution: true,
            enable_json_schema: true,
            stop_words,
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Optimized prompt result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPrompt {
    /// The optimized prompt text
    pub text: String,
    /// Original length (characters)
    pub original_length: usize,
    /// Compressed length (characters)
    pub compressed_length: usize,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Estimated token count
    pub estimated_tokens: usize,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// Applied optimizations
    pub optimizations_applied: Vec<String>,
    /// JSON schema for structured output (if enabled)
    pub output_schema: Option<serde_json::Value>,
}

/// An entity extracted from the prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub kg_reference: Option<String>,
    pub start_pos: usize,
    pub end_pos: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Number,
    Technical,
    Concept,
    Unknown,
}

/// Knowledge graph reference for entity substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KGReference {
    pub entity_id: String,
    pub label: String,
    pub entity_type: EntityType,
    pub description: Option<String>,
}

// ============================================================================
// Prompt Optimizer Service
// ============================================================================

pub struct PromptOptimizer {
    config: OptimizerConfig,
    /// Knowledge graph entity cache
    kg_cache: HashMap<String, KGReference>,
}

impl PromptOptimizer {
    pub fn new(config: OptimizerConfig) -> Self {
        Self {
            config,
            kg_cache: HashMap::new(),
        }
    }

    /// Optimize a prompt for inference
    pub fn optimize(&self, prompt: &str, context: Option<&str>) -> Result<OptimizedPrompt> {
        let original_length = prompt.len() + context.map(|c| c.len()).unwrap_or(0);
        let mut optimizations = Vec::new();

        // Step 1: Extract entities
        let entities = self.extract_entities(prompt);
        if !entities.is_empty() {
            optimizations.push(format!("Extracted {} entities", entities.len()));
        }

        // Step 2: Compress context if provided
        let compressed_context = if let Some(ctx) = context {
            let compressed = self.compress_context(ctx);
            if compressed.len() < ctx.len() {
                optimizations.push(format!(
                    "Context compressed: {} -> {} chars",
                    ctx.len(),
                    compressed.len()
                ));
            }
            Some(compressed)
        } else {
            None
        };

        // Step 3: Build optimized prompt
        let mut optimized_text = String::new();

        // Add compressed context
        if let Some(ctx) = compressed_context {
            if !ctx.is_empty() {
                optimized_text.push_str("Context:\n");
                optimized_text.push_str(&ctx);
                optimized_text.push_str("\n\n");
            }
        }

        // Add main prompt (with KG substitutions if enabled)
        let processed_prompt = if self.config.enable_kg_substitution {
            self.substitute_kg_references(prompt, &entities)
        } else {
            prompt.to_string()
        };
        optimized_text.push_str("Task:\n");
        optimized_text.push_str(&processed_prompt);

        // Step 4: Generate JSON schema if enabled
        let output_schema = if self.config.enable_json_schema {
            Some(self.generate_output_schema(&entities))
        } else {
            None
        };

        if output_schema.is_some() {
            optimized_text.push_str("\n\nRespond in JSON format matching the schema.");
            optimizations.push("Added JSON schema output format".to_string());
        }

        let compressed_length = optimized_text.len();
        let compression_ratio = if original_length > 0 {
            1.0 - (compressed_length as f64 / original_length as f64)
        } else {
            0.0
        };

        let estimated_tokens = self.estimate_tokens(&optimized_text);

        Ok(OptimizedPrompt {
            text: optimized_text,
            original_length,
            compressed_length,
            compression_ratio,
            estimated_tokens,
            entities,
            optimizations_applied: optimizations,
            output_schema,
        })
    }

    /// Extract entities from text
    fn extract_entities(&self, text: &str) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();

        // Simple rule-based entity extraction (for MVP)
        // In production, use NER models

        // Extract capitalized phrases (potential names/orgs)
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut i = 0;
        while i < words.len() {
            let word = words[i];
            if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                // Check for multi-word capitalized phrases
                let mut phrase = word.to_string();
                let start_pos = text.find(word).unwrap_or(0);
                let mut j = i + 1;

                while j < words.len() {
                    let next = words[j];
                    if next.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                        phrase.push(' ');
                        phrase.push_str(next);
                        j += 1;
                    } else {
                        break;
                    }
                }

                let entity_type = self.infer_entity_type(&phrase);
                let kg_ref = self.lookup_kg_reference(&phrase);

                entities.push(ExtractedEntity {
                    text: phrase.clone(),
                    entity_type,
                    kg_reference: kg_ref.map(|r| r.entity_id),
                    start_pos,
                    end_pos: start_pos + phrase.len(),
                });

                i = j;
            } else {
                i += 1;
            }
        }

        // Extract numbers and dates
        for (i, word) in words.iter().enumerate() {
            if word.chars().all(|c| c.is_numeric() || c == '.' || c == ',') {
                let start_pos = text.find(word).unwrap_or(0);
                entities.push(ExtractedEntity {
                    text: word.to_string(),
                    entity_type: EntityType::Number,
                    kg_reference: None,
                    start_pos,
                    end_pos: start_pos + word.len(),
                });
            }
        }

        entities
    }

    /// Infer entity type from text
    fn infer_entity_type(&self, text: &str) -> EntityType {
        let lower = text.to_lowercase();

        // Location indicators
        if lower.ends_with("city")
            || lower.ends_with("country")
            || lower.ends_with("state")
            || lower.contains("street")
        {
            return EntityType::Location;
        }

        // Organization indicators
        if lower.ends_with("inc")
            || lower.ends_with("corp")
            || lower.ends_with("llc")
            || lower.ends_with("ltd")
            || lower.contains("company")
        {
            return EntityType::Organization;
        }

        // Technical terms
        if lower.contains("api")
            || lower.contains("sdk")
            || lower.contains("http")
            || lower.contains("json")
        {
            return EntityType::Technical;
        }

        // Date patterns
        if lower.contains("january")
            || lower.contains("february")
            || lower.contains("march")
            || lower.contains("2024")
            || lower.contains("2025")
        {
            return EntityType::Date;
        }

        EntityType::Unknown
    }

    /// Compress context by removing redundancy
    fn compress_context(&self, context: &str) -> String {
        let mut result = Vec::new();
        let sentences: Vec<&str> = context.split('.').collect();

        // Score each sentence by information density
        let mut seen_content = HashSet::new();

        for sentence in sentences {
            let trimmed = sentence.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Extract content words (non-stop words)
            let content_words: Vec<String> = trimmed
                .to_lowercase()
                .split_whitespace()
                .filter(|w| !self.config.stop_words.contains(*w))
                .map(|s| s.to_string())
                .collect();

            // Check for novelty
            let new_words: Vec<_> = content_words
                .iter()
                .filter(|w| !seen_content.contains(*w))
                .cloned()
                .collect();

            // Keep sentence if it has enough new information
            if new_words.len() as f64 / content_words.len().max(1) as f64 > 0.3 {
                result.push(trimmed);
                for word in content_words {
                    seen_content.insert(word);
                }
            }
        }

        result.join(". ")
    }

    /// Substitute entity references with KG IDs
    fn substitute_kg_references(&self, text: &str, entities: &[ExtractedEntity]) -> String {
        let mut result = text.to_string();

        // Sort entities by position (descending) to avoid offset issues
        let mut sorted_entities = entities.to_vec();
        sorted_entities.sort_by(|a, b| b.start_pos.cmp(&a.start_pos));

        for entity in sorted_entities {
            if let Some(ref kg_id) = entity.kg_reference {
                // Replace with compact reference format
                let replacement = format!("[{}:{}]", kg_id, entity.text);
                if result.len() > entity.end_pos {
                    result.replace_range(entity.start_pos..entity.end_pos, &replacement);
                }
            }
        }

        result
    }

    /// Lookup entity in knowledge graph cache
    fn lookup_kg_reference(&self, entity_text: &str) -> Option<&KGReference> {
        let normalized = entity_text.to_lowercase();
        self.kg_cache.get(&normalized)
    }

    /// Register a KG entity for reference substitution
    pub fn register_kg_entity(&mut self, entity: KGReference) {
        let key = entity.label.to_lowercase();
        self.kg_cache.insert(key, entity);
    }

    /// Generate JSON schema for structured output
    fn generate_output_schema(&self, entities: &[ExtractedEntity]) -> serde_json::Value {
        let mut properties = serde_json::Map::new();

        // Always include answer field
        properties.insert(
            "answer".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "The main answer to the query"
            }),
        );

        // Add confidence
        properties.insert(
            "confidence".to_string(),
            serde_json::json!({
                "type": "number",
                "minimum": 0,
                "maximum": 1,
                "description": "Confidence score (0-1)"
            }),
        );

        // Add entity-specific fields
        for entity in entities {
            let field_name = entity
                .text
                .to_lowercase()
                .replace(' ', "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();

            if !field_name.is_empty() && !properties.contains_key(&field_name) {
                properties.insert(
                    field_name,
                    serde_json::json!({
                        "type": "string",
                        "description": format!("Information about {}", entity.text)
                    }),
                );
            }
        }

        // Add reasoning field
        properties.insert(
            "reasoning".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Step-by-step reasoning process"
            }),
        );

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": ["answer", "confidence"]
        })
    }

    /// Estimate token count (approximate: 1 token ≈ 4 chars for English)
    fn estimate_tokens(&self, text: &str) -> usize {
        // More accurate estimation
        let word_count = text.split_whitespace().count();
        let char_count = text.len();

        // Roughly 0.75 tokens per word + punctuation overhead
        (word_count as f64 * 1.3 + char_count as f64 * 0.1) as usize
    }

    /// Check if prompt exceeds token budget
    pub fn exceeds_budget(&self, prompt: &OptimizedPrompt) -> bool {
        prompt.estimated_tokens > self.config.max_input_tokens
    }

    /// Truncate prompt to fit budget
    pub fn truncate_to_budget(&self, prompt: &mut OptimizedPrompt) {
        if !self.exceeds_budget(prompt) {
            return;
        }

        // Simple truncation strategy: remove from the middle
        let target_chars = (self.config.max_input_tokens * 4) as usize;
        if prompt.text.len() > target_chars {
            let half = target_chars / 2;
            let start = &prompt.text[..half];
            let end = &prompt.text[prompt.text.len() - half..];
            prompt.text = format!("{}\n...[truncated]...\n{}", start, end);
            prompt.estimated_tokens = self.estimate_tokens(&prompt.text);
            prompt
                .optimizations_applied
                .push("Truncated to fit token budget".to_string());
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_extraction() {
        let optimizer = PromptOptimizer::new(OptimizerConfig::default());
        let text = "What is the capital of France? The Eiffel Tower is located in Paris.";
        let entities = optimizer.extract_entities(text);

        assert!(!entities.is_empty());
        let entity_texts: Vec<_> = entities.iter().map(|e| e.text.as_str()).collect();
        assert!(entity_texts.contains(&"France"));
        assert!(entity_texts.contains(&"Eiffel Tower"));
        assert!(entity_texts.contains(&"Paris"));
    }

    #[test]
    fn test_context_compression() {
        let optimizer = PromptOptimizer::new(OptimizerConfig::default());

        let context = "The quick brown fox jumps over the lazy dog. \
                       The quick brown fox is very fast. \
                       The lazy dog is sleeping. \
                       A new unrelated fact about cats.";

        let compressed = optimizer.compress_context(context);
        assert!(compressed.len() < context.len());
    }

    #[test]
    fn test_token_estimation() {
        let optimizer = PromptOptimizer::new(OptimizerConfig::default());

        let short_text = "Hello world";
        let long_text = "This is a much longer text that contains many more words and should have a higher token count.";

        assert!(optimizer.estimate_tokens(short_text) < optimizer.estimate_tokens(long_text));
    }

    #[test]
    fn test_optimize_prompt() {
        let optimizer = PromptOptimizer::new(OptimizerConfig::default());

        let result = optimizer
            .optimize(
                "What is the population of New York City?",
                Some("New York City is the largest city in the United States. It has a very large population. The city is known for its landmarks."),
            )
            .unwrap();

        assert!(!result.text.is_empty());
        assert!(result.compression_ratio >= 0.0);
        assert!(!result.entities.is_empty());
    }

    #[test]
    fn test_json_schema_generation() {
        let optimizer = PromptOptimizer::new(OptimizerConfig::default());
        let entities = vec![ExtractedEntity {
            text: "Paris".to_string(),
            entity_type: EntityType::Location,
            kg_reference: None,
            start_pos: 0,
            end_pos: 5,
        }];

        let schema = optimizer.generate_output_schema(&entities);
        assert!(schema.get("properties").is_some());
        assert!(schema["properties"].get("answer").is_some());
        assert!(schema["properties"].get("paris").is_some());
    }
}

