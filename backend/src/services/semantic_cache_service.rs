//! TRO Semantic Cache Service
//!
//! Implements intelligent caching for the reasoning layer:
//! - SBERT-based semantic similarity for query matching
//! - Redis backend for distributed cache storage
//! - Signed cached responses for trustworthiness
//! - TTL-based expiration with category-aware policies

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Configuration
// ============================================================================

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Default TTL for cached entries (seconds)
    pub default_ttl_secs: u64,
    /// Similarity threshold for cache hits (0.0 - 1.0)
    pub similarity_threshold: f64,
    /// Maximum cache entries per category
    pub max_entries_per_category: usize,
    /// Enable local in-memory cache layer
    pub enable_local_cache: bool,
    /// Local cache size limit
    pub local_cache_size: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            default_ttl_secs: 86400, // 24 hours
            similarity_threshold: 0.95,
            max_entries_per_category: 10000,
            enable_local_cache: true,
            local_cache_size: 1000,
        }
    }
}

/// Category-specific TTL policies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CacheCategory {
    /// Factual knowledge (long TTL)
    Factual,
    /// Price/market data (short TTL)
    PriceData,
    /// Code/technical content (medium TTL)
    Technical,
    /// General Q&A (medium TTL)
    GeneralQA,
    /// Time-sensitive info (very short TTL)
    TimeSensitive,
}

impl CacheCategory {
    pub fn ttl_seconds(&self) -> u64 {
        match self {
            CacheCategory::Factual => 7 * 24 * 3600,     // 7 days
            CacheCategory::PriceData => 300,             // 5 minutes
            CacheCategory::Technical => 24 * 3600,       // 1 day
            CacheCategory::GeneralQA => 12 * 3600,       // 12 hours
            CacheCategory::TimeSensitive => 60,          // 1 minute
        }
    }

    /// Infer category from query content
    pub fn infer_from_query(query: &str) -> Self {
        let query_lower = query.to_lowercase();

        if query_lower.contains("price")
            || query_lower.contains("cost")
            || query_lower.contains("market")
            || query_lower.contains("trading")
        {
            return CacheCategory::PriceData;
        }

        if query_lower.contains("now")
            || query_lower.contains("today")
            || query_lower.contains("current")
            || query_lower.contains("latest")
        {
            return CacheCategory::TimeSensitive;
        }

        if query_lower.contains("code")
            || query_lower.contains("function")
            || query_lower.contains("program")
            || query_lower.contains("implement")
        {
            return CacheCategory::Technical;
        }

        if query_lower.contains("capital of")
            || query_lower.contains("who is")
            || query_lower.contains("what is")
            || query_lower.contains("define")
        {
            return CacheCategory::Factual;
        }

        CacheCategory::GeneralQA
    }
}

// ============================================================================
// Cache Entry Types
// ============================================================================

/// A cached reasoning result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Original query
    pub query: String,
    /// Query embedding (simplified: hash-based for MVP)
    pub query_hash: String,
    /// Cached response
    pub response: String,
    /// Response hash for verification
    pub response_hash: String,
    /// Node that generated this response
    pub node_pubkey: String,
    /// Signature from the node (hex-encoded)
    pub signature: String,
    /// Model used for generation
    pub model_used: String,
    /// Confidence score (0-10000 bps)
    pub confidence_bps: u16,
    /// Cache category
    pub category: CacheCategory,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: DateTime<Utc>,
    /// Number of times this entry was hit
    pub hit_count: u64,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl CacheEntry {
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn ttl_remaining(&self) -> i64 {
        (self.expires_at - Utc::now()).num_seconds()
    }
}

/// Cache lookup result
#[derive(Debug, Clone)]
pub struct CacheLookupResult {
    pub hit: bool,
    pub entry: Option<CacheEntry>,
    pub similarity_score: f64,
    pub lookup_time_ms: u64,
}

// ============================================================================
// Semantic Cache Service
// ============================================================================

/// Main semantic cache service
pub struct SemanticCacheService {
    config: CacheConfig,
    redis: Option<redis::aio::ConnectionManager>,
    /// Local in-memory cache (LRU)
    local_cache: RwLock<HashMap<String, CacheEntry>>,
    /// Cache statistics
    stats: RwLock<CacheStats>,
}

#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub total_lookups: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub local_hits: u64,
    pub redis_hits: u64,
    pub entries_created: u64,
    pub entries_expired: u64,
}

impl SemanticCacheService {
    pub async fn new(config: CacheConfig) -> Result<Self> {
        // Try to connect to Redis
        let redis = match redis::Client::open(config.redis_url.clone()) {
            Ok(client) => match redis::aio::ConnectionManager::new(client).await {
                Ok(conn) => {
                    info!("Connected to Redis at {}", config.redis_url);
                    Some(conn)
                }
                Err(e) => {
                    warn!("Failed to connect to Redis: {}. Using local cache only.", e);
                    None
                }
            },
            Err(e) => {
                warn!("Invalid Redis URL: {}. Using local cache only.", e);
                None
            }
        };

        Ok(Self {
            config,
            redis,
            local_cache: RwLock::new(HashMap::new()),
            stats: RwLock::new(CacheStats::default()),
        })
    }

    /// Look up a query in the cache
    pub async fn lookup(&self, query: &str) -> CacheLookupResult {
        let start = std::time::Instant::now();
        let mut stats = self.stats.write().await;
        stats.total_lookups += 1;

        let query_hash = self.compute_query_hash(query);

        // Try local cache first
        if self.config.enable_local_cache {
            let local = self.local_cache.read().await;
            if let Some(entry) = local.get(&query_hash) {
                if !entry.is_expired() {
                    stats.cache_hits += 1;
                    stats.local_hits += 1;
                    return CacheLookupResult {
                        hit: true,
                        entry: Some(entry.clone()),
                        similarity_score: 1.0, // Exact hash match
                        lookup_time_ms: start.elapsed().as_millis() as u64,
                    };
                }
            }
        }

        // Try Redis cache
        if let Some(ref redis) = self.redis {
            match self.lookup_redis(redis.clone(), &query_hash).await {
                Ok(Some(entry)) => {
                    if !entry.is_expired() {
                        stats.cache_hits += 1;
                        stats.redis_hits += 1;

                        // Promote to local cache
                        if self.config.enable_local_cache {
                            drop(stats); // Release write lock
                            let mut local = self.local_cache.write().await;
                            self.evict_if_needed(&mut local);
                            local.insert(query_hash.clone(), entry.clone());
                        }

                        return CacheLookupResult {
                            hit: true,
                            entry: Some(entry),
                            similarity_score: 1.0,
                            lookup_time_ms: start.elapsed().as_millis() as u64,
                        };
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    warn!("Redis lookup failed: {}", e);
                }
            }
        }

        // Try semantic similarity search (simplified for MVP)
        // In production, this would use vector embeddings
        if let Some(entry) = self.semantic_search(query).await {
            let similarity = self.compute_similarity(query, &entry.query);
            if similarity >= self.config.similarity_threshold {
                stats.cache_hits += 1;
                return CacheLookupResult {
                    hit: true,
                    entry: Some(entry),
                    similarity_score: similarity,
                    lookup_time_ms: start.elapsed().as_millis() as u64,
                };
            }
        }

        stats.cache_misses += 1;
        CacheLookupResult {
            hit: false,
            entry: None,
            similarity_score: 0.0,
            lookup_time_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Store a new cache entry
    pub async fn store(
        &self,
        query: &str,
        response: &str,
        node_pubkey: &str,
        signature: &str,
        model_used: &str,
        confidence_bps: u16,
        category: Option<CacheCategory>,
    ) -> Result<()> {
        let category = category.unwrap_or_else(|| CacheCategory::infer_from_query(query));
        let ttl_secs = category.ttl_seconds();

        let query_hash = self.compute_query_hash(query);
        let response_hash = self.compute_response_hash(response);

        let entry = CacheEntry {
            query: query.to_string(),
            query_hash: query_hash.clone(),
            response: response.to_string(),
            response_hash,
            node_pubkey: node_pubkey.to_string(),
            signature: signature.to_string(),
            model_used: model_used.to_string(),
            confidence_bps,
            category,
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::seconds(ttl_secs as i64),
            hit_count: 0,
            metadata: HashMap::new(),
        };

        // Store in Redis
        if let Some(ref redis) = self.redis {
            self.store_redis(redis.clone(), &entry, ttl_secs).await?;
        }

        // Store in local cache
        if self.config.enable_local_cache {
            let mut local = self.local_cache.write().await;
            self.evict_if_needed(&mut local);
            local.insert(query_hash, entry);
        }

        let mut stats = self.stats.write().await;
        stats.entries_created += 1;

        Ok(())
    }

    /// Invalidate a cache entry
    pub async fn invalidate(&self, query: &str) -> Result<()> {
        let query_hash = self.compute_query_hash(query);

        // Remove from local cache
        {
            let mut local = self.local_cache.write().await;
            local.remove(&query_hash);
        }

        // Remove from Redis
        if let Some(ref redis) = self.redis {
            let mut conn = redis.clone();
            let key = format!("tro:cache:{}", query_hash);
            let _: () = conn.del(&key).await?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// Compute cache hit rate
    pub async fn hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.total_lookups == 0 {
            0.0
        } else {
            (stats.cache_hits as f64 / stats.total_lookups as f64) * 100.0
        }
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    fn compute_query_hash(&self, query: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(query.trim().to_lowercase().as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    fn compute_response_hash(&self, response: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(response.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Simplified similarity computation (for MVP)
    /// In production, use SBERT embeddings and cosine similarity
    fn compute_similarity(&self, query1: &str, query2: &str) -> f64 {
        let q1_words: std::collections::HashSet<_> = query1
            .to_lowercase()
            .split_whitespace()
            .collect();
        let q2_words: std::collections::HashSet<_> = query2
            .to_lowercase()
            .split_whitespace()
            .collect();

        if q1_words.is_empty() || q2_words.is_empty() {
            return 0.0;
        }

        let intersection = q1_words.intersection(&q2_words).count();
        let union = q1_words.union(&q2_words).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    async fn lookup_redis(
        &self,
        mut conn: redis::aio::ConnectionManager,
        query_hash: &str,
    ) -> Result<Option<CacheEntry>> {
        let key = format!("tro:cache:{}", query_hash);
        let data: Option<String> = conn.get(&key).await?;

        match data {
            Some(json) => {
                let entry: CacheEntry = serde_json::from_str(&json)?;
                Ok(Some(entry))
            }
            None => Ok(None),
        }
    }

    async fn store_redis(
        &self,
        mut conn: redis::aio::ConnectionManager,
        entry: &CacheEntry,
        ttl_secs: u64,
    ) -> Result<()> {
        let key = format!("tro:cache:{}", entry.query_hash);
        let json = serde_json::to_string(entry)?;
        let _: () = conn.set_ex(&key, json, ttl_secs).await?;
        Ok(())
    }

    /// Semantic search through local cache (simplified)
    async fn semantic_search(&self, query: &str) -> Option<CacheEntry> {
        let local = self.local_cache.read().await;
        let mut best_match: Option<(f64, CacheEntry)> = None;

        for entry in local.values() {
            if entry.is_expired() {
                continue;
            }

            let similarity = self.compute_similarity(query, &entry.query);
            if similarity >= self.config.similarity_threshold {
                match &best_match {
                    None => best_match = Some((similarity, entry.clone())),
                    Some((best_sim, _)) if similarity > *best_sim => {
                        best_match = Some((similarity, entry.clone()));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(_, entry)| entry)
    }

    /// Evict oldest entries if cache is full
    fn evict_if_needed(&self, cache: &mut HashMap<String, CacheEntry>) {
        if cache.len() >= self.config.local_cache_size {
            // Simple eviction: remove expired entries first
            cache.retain(|_, entry| !entry.is_expired());

            // If still full, remove oldest entries
            if cache.len() >= self.config.local_cache_size {
                let mut entries: Vec<_> = cache.iter().collect();
                entries.sort_by_key(|(_, e)| e.created_at);

                // Remove oldest 10%
                let to_remove = cache.len() / 10;
                for (key, _) in entries.iter().take(to_remove) {
                    cache.remove(*key);
                }
            }
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
    fn test_category_inference() {
        assert_eq!(
            CacheCategory::infer_from_query("What is the current price of Bitcoin?"),
            CacheCategory::PriceData
        );

        assert_eq!(
            CacheCategory::infer_from_query("What is the capital of France?"),
            CacheCategory::Factual
        );

        assert_eq!(
            CacheCategory::infer_from_query("Write a Python function to sort a list"),
            CacheCategory::Technical
        );

        assert_eq!(
            CacheCategory::infer_from_query("What is happening today?"),
            CacheCategory::TimeSensitive
        );
    }

    #[test]
    fn test_ttl_ordering() {
        assert!(CacheCategory::TimeSensitive.ttl_seconds() < CacheCategory::PriceData.ttl_seconds());
        assert!(CacheCategory::PriceData.ttl_seconds() < CacheCategory::GeneralQA.ttl_seconds());
        assert!(CacheCategory::GeneralQA.ttl_seconds() < CacheCategory::Factual.ttl_seconds());
    }

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let config = CacheConfig {
            redis_url: "redis://invalid:6379".to_string(), // Will fail, use local only
            enable_local_cache: true,
            ..Default::default()
        };

        let cache = SemanticCacheService::new(config).await.unwrap();

        // Store entry
        cache
            .store(
                "What is 2+2?",
                "4",
                "test_node",
                "test_sig",
                "test_model",
                9000,
                Some(CacheCategory::Factual),
            )
            .await
            .unwrap();

        // Lookup should hit
        let result = cache.lookup("What is 2+2?").await;
        assert!(result.hit);
        assert_eq!(result.entry.unwrap().response, "4");

        // Different query should miss
        let result = cache.lookup("What is 3+3?").await;
        assert!(!result.hit);
    }
}

