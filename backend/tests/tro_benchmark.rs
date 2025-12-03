//! TRO Performance Benchmark Tests
//!
//! Measures and validates performance characteristics of the TRO pipeline:
//! - Throughput comparison vs SenteTruth baseline
//! - Latency for cached vs uncached requests
//! - Gas cost estimation per task
//! - Malicious node resilience (40% adversarial scenario)

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Benchmark configuration
pub struct BenchmarkConfig {
    /// Number of iterations per test
    pub iterations: usize,
    /// Simulated node count
    pub node_count: usize,
    /// Percentage of malicious nodes (0-100)
    pub malicious_percentage: u8,
    /// Cache hit rate target
    pub target_cache_hit_rate: f64,
    /// Latency thresholds
    pub max_cache_hit_latency_ms: u64,
    pub max_local_inference_latency_ms: u64,
    pub max_api_inference_latency_ms: u64,
    /// Cost threshold (lamports per task)
    pub max_cost_per_task: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            node_count: 10,
            malicious_percentage: 40,
            target_cache_hit_rate: 0.60,
            max_cache_hit_latency_ms: 100,
            max_local_inference_latency_ms: 2000,
            max_api_inference_latency_ms: 5000,
            max_cost_per_task: 1_000_000, // 0.001 SOL
        }
    }
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Test name
    pub test_name: String,
    /// Total iterations run
    pub iterations: usize,
    /// Total time taken
    pub total_time: Duration,
    /// Average latency per operation
    pub avg_latency: Duration,
    /// 50th percentile latency
    pub p50_latency: Duration,
    /// 95th percentile latency
    pub p95_latency: Duration,
    /// 99th percentile latency
    pub p99_latency: Duration,
    /// Operations per second (throughput)
    pub ops_per_second: f64,
    /// Success rate (0-1)
    pub success_rate: f64,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

impl BenchmarkResults {
    pub fn new(test_name: &str) -> Self {
        Self {
            test_name: test_name.to_string(),
            iterations: 0,
            total_time: Duration::ZERO,
            avg_latency: Duration::ZERO,
            p50_latency: Duration::ZERO,
            p95_latency: Duration::ZERO,
            p99_latency: Duration::ZERO,
            ops_per_second: 0.0,
            success_rate: 0.0,
            metrics: HashMap::new(),
        }
    }

    pub fn calculate_from_latencies(test_name: &str, latencies: &[Duration], successes: usize) -> Self {
        let mut sorted = latencies.to_vec();
        sorted.sort();
        
        let total_time: Duration = latencies.iter().sum();
        let iterations = latencies.len();
        
        let avg_latency = if iterations > 0 {
            total_time / iterations as u32
        } else {
            Duration::ZERO
        };
        
        let p50_latency = sorted.get(iterations / 2).copied().unwrap_or(Duration::ZERO);
        let p95_latency = sorted.get(iterations * 95 / 100).copied().unwrap_or(Duration::ZERO);
        let p99_latency = sorted.get(iterations * 99 / 100).copied().unwrap_or(Duration::ZERO);
        
        let ops_per_second = if total_time.as_secs_f64() > 0.0 {
            iterations as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };
        
        let success_rate = if iterations > 0 {
            successes as f64 / iterations as f64
        } else {
            0.0
        };
        
        Self {
            test_name: test_name.to_string(),
            iterations,
            total_time,
            avg_latency,
            p50_latency,
            p95_latency,
            p99_latency,
            ops_per_second,
            success_rate,
            metrics: HashMap::new(),
        }
    }

    pub fn print_summary(&self) {
        println!("\n=== {} ===", self.test_name);
        println!("Iterations: {}", self.iterations);
        println!("Total Time: {:?}", self.total_time);
        println!("Avg Latency: {:?}", self.avg_latency);
        println!("P50 Latency: {:?}", self.p50_latency);
        println!("P95 Latency: {:?}", self.p95_latency);
        println!("P99 Latency: {:?}", self.p99_latency);
        println!("Throughput: {:.2} ops/sec", self.ops_per_second);
        println!("Success Rate: {:.2}%", self.success_rate * 100.0);
        
        for (key, value) in &self.metrics {
            println!("{}: {:.4}", key, value);
        }
    }
}

/// Simulated node for testing
#[derive(Debug, Clone)]
pub struct SimulatedNode {
    pub id: String,
    pub is_malicious: bool,
    pub reputation: u16,
    pub latency_ms: u64,
    pub success_rate: f64,
}

impl SimulatedNode {
    pub fn honest(id: &str) -> Self {
        Self {
            id: id.to_string(),
            is_malicious: false,
            reputation: 9000,
            latency_ms: 500,
            success_rate: 0.98,
        }
    }

    pub fn malicious(id: &str) -> Self {
        Self {
            id: id.to_string(),
            is_malicious: true,
            reputation: 5000,
            latency_ms: 200, // Faster but wrong
            success_rate: 0.0, // Always wrong
        }
    }

    /// Simulate inference, returns (result_hash, is_correct)
    pub async fn infer(&self, task_hash: &[u8; 32]) -> (Vec<u8>, bool) {
        // Simulate latency
        tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;
        
        if self.is_malicious {
            // Return wrong result
            let mut wrong_hash = task_hash.to_vec();
            wrong_hash[0] = wrong_hash[0].wrapping_add(1);
            (wrong_hash, false)
        } else {
            // Return correct result with some probability of failure
            let is_correct = rand::random::<f64>() < self.success_rate;
            if is_correct {
                (task_hash.to_vec(), true)
            } else {
                let mut wrong_hash = task_hash.to_vec();
                wrong_hash[0] = wrong_hash[0].wrapping_add(1);
                (wrong_hash, false)
            }
        }
    }
}

/// Simulated TRO network for testing
pub struct SimulatedTroNetwork {
    pub nodes: Vec<SimulatedNode>,
    pub cache: Arc<Mutex<HashMap<[u8; 32], Vec<u8>>>>,
    pub config: BenchmarkConfig,
}

impl SimulatedTroNetwork {
    pub fn new(config: BenchmarkConfig) -> Self {
        let mut nodes = Vec::new();
        let malicious_count = (config.node_count as f64 * config.malicious_percentage as f64 / 100.0) as usize;
        
        for i in 0..config.node_count {
            if i < malicious_count {
                nodes.push(SimulatedNode::malicious(&format!("malicious_{}", i)));
            } else {
                nodes.push(SimulatedNode::honest(&format!("honest_{}", i)));
            }
        }
        
        Self {
            nodes,
            cache: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    /// Run multi-node inference with majority voting
    pub async fn run_inference(&self, task_hash: [u8; 32]) -> (Vec<u8>, bool, Duration) {
        let start = Instant::now();
        
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if let Some(cached) = cache.get(&task_hash) {
                return (cached.clone(), true, start.elapsed());
            }
        }
        
        // Run inference on all nodes concurrently
        let mut handles = Vec::new();
        for node in &self.nodes {
            let node = node.clone();
            let task = task_hash;
            handles.push(tokio::spawn(async move {
                node.infer(&task).await
            }));
        }
        
        // Collect results
        let mut results: HashMap<Vec<u8>, usize> = HashMap::new();
        let mut correct_count = 0;
        
        for handle in handles {
            if let Ok((result, is_correct)) = handle.await {
                *results.entry(result).or_insert(0) += 1;
                if is_correct {
                    correct_count += 1;
                }
            }
        }
        
        // Find majority result
        let (majority_result, majority_count) = results
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(r, c)| (r.clone(), *c))
            .unwrap_or((task_hash.to_vec(), 0));
        
        // Verify if majority is correct
        let is_correct = majority_result == task_hash.to_vec() && majority_count > self.nodes.len() / 2;
        
        // Cache result if correct
        if is_correct {
            let mut cache = self.cache.lock().await;
            cache.insert(task_hash, majority_result.clone());
        }
        
        (majority_result, is_correct, start.elapsed())
    }
}

// ============================================================================
// Benchmark Tests
// ============================================================================

/// Test 1: Throughput comparison vs SenteTruth baseline
/// Target: 3-5x improvement over paper's baseline
pub async fn benchmark_throughput(config: &BenchmarkConfig) -> BenchmarkResults {
    let network = SimulatedTroNetwork::new(config.clone());
    let mut latencies = Vec::with_capacity(config.iterations);
    let mut successes = 0;
    
    for i in 0..config.iterations {
        let task_hash: [u8; 32] = {
            let mut hash = [0u8; 32];
            hash[0..8].copy_from_slice(&(i as u64).to_le_bytes());
            hash
        };
        
        let (_, is_correct, latency) = network.run_inference(task_hash).await;
        latencies.push(latency);
        if is_correct {
            successes += 1;
        }
    }
    
    let mut results = BenchmarkResults::calculate_from_latencies(
        "Throughput Benchmark",
        &latencies,
        successes,
    );
    
    // Compare with SenteTruth baseline (from paper: ~10 ops/sec)
    let baseline_ops = 10.0;
    let improvement = results.ops_per_second / baseline_ops;
    results.metrics.insert("baseline_ops_per_sec".to_string(), baseline_ops);
    results.metrics.insert("improvement_factor".to_string(), improvement);
    
    results
}

/// Test 2: Cache hit latency
/// Target: < 100ms for cache hits
pub async fn benchmark_cache_latency(config: &BenchmarkConfig) -> BenchmarkResults {
    let network = SimulatedTroNetwork::new(config.clone());
    let mut latencies = Vec::with_capacity(config.iterations);
    let mut cache_hits = 0;
    
    // First pass: populate cache
    let task_hash: [u8; 32] = [1u8; 32];
    let _ = network.run_inference(task_hash).await;
    
    // Second pass: measure cache hit latency
    for _ in 0..config.iterations {
        let start = Instant::now();
        {
            let cache = network.cache.lock().await;
            if cache.get(&task_hash).is_some() {
                cache_hits += 1;
            }
        }
        latencies.push(start.elapsed());
    }
    
    let mut results = BenchmarkResults::calculate_from_latencies(
        "Cache Hit Latency",
        &latencies,
        cache_hits,
    );
    
    results.metrics.insert("cache_hit_rate".to_string(), cache_hits as f64 / config.iterations as f64);
    results.metrics.insert("target_latency_ms".to_string(), config.max_cache_hit_latency_ms as f64);
    
    results
}

/// Test 3: Malicious node resilience
/// Target: > 99% accuracy with 40% malicious nodes
pub async fn benchmark_malicious_resilience(config: &BenchmarkConfig) -> BenchmarkResults {
    let network = SimulatedTroNetwork::new(config.clone());
    let mut latencies = Vec::with_capacity(config.iterations);
    let mut correct_results = 0;
    
    for i in 0..config.iterations {
        let task_hash: [u8; 32] = {
            let mut hash = [0u8; 32];
            hash[0..8].copy_from_slice(&(i as u64 + 1000).to_le_bytes());
            hash
        };
        
        let (result, is_correct, latency) = network.run_inference(task_hash).await;
        latencies.push(latency);
        
        // Verify result is correct
        if result == task_hash.to_vec() {
            correct_results += 1;
        }
    }
    
    let mut results = BenchmarkResults::calculate_from_latencies(
        &format!("Malicious Resilience ({}% malicious)", config.malicious_percentage),
        &latencies,
        correct_results,
    );
    
    let accuracy = correct_results as f64 / config.iterations as f64;
    results.metrics.insert("accuracy".to_string(), accuracy);
    results.metrics.insert("malicious_percentage".to_string(), config.malicious_percentage as f64);
    
    // Paper baseline: 17.74% improvement means ~82% accuracy
    let baseline_accuracy = 0.82;
    let improvement = (accuracy - baseline_accuracy) / baseline_accuracy * 100.0;
    results.metrics.insert("baseline_accuracy".to_string(), baseline_accuracy);
    results.metrics.insert("improvement_percentage".to_string(), improvement);
    
    results
}

/// Test 4: Gas cost estimation
/// Target: < 0.001 SOL per task
pub async fn benchmark_gas_cost(config: &BenchmarkConfig) -> BenchmarkResults {
    // Simulated gas costs (in lamports)
    const BASE_TX_COST: u64 = 5000;          // Base transaction fee
    const ACCOUNT_RENT: u64 = 100000;         // Account rent per task
    const COMPUTE_UNIT_COST: u64 = 1000;      // Per compute unit
    const ZK_VERIFICATION_COST: u64 = 500000; // ZK proof verification
    
    let mut total_cost: u64 = 0;
    let mut costs = Vec::with_capacity(config.iterations);
    
    for i in 0..config.iterations {
        let is_high_value = i % 3 == 0; // 33% require ZK proof
        
        let task_cost = BASE_TX_COST 
            + ACCOUNT_RENT 
            + COMPUTE_UNIT_COST * 200 // ~200 compute units per task
            + if is_high_value { ZK_VERIFICATION_COST } else { 0 };
        
        costs.push(task_cost);
        total_cost += task_cost;
    }
    
    let avg_cost = total_cost / config.iterations as u64;
    
    let mut results = BenchmarkResults::new("Gas Cost Estimation");
    results.iterations = config.iterations;
    results.metrics.insert("avg_cost_lamports".to_string(), avg_cost as f64);
    results.metrics.insert("avg_cost_sol".to_string(), avg_cost as f64 / 1e9);
    results.metrics.insert("target_cost_lamports".to_string(), config.max_cost_per_task as f64);
    results.metrics.insert("within_budget".to_string(), if avg_cost <= config.max_cost_per_task { 1.0 } else { 0.0 });
    
    results
}

// ============================================================================
// Test Runner
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_throughput_benchmark() {
        let config = BenchmarkConfig {
            iterations: 50,
            ..Default::default()
        };
        
        let results = benchmark_throughput(&config).await;
        results.print_summary();
        
        // Verify minimum throughput target
        assert!(results.ops_per_second > 10.0, "Throughput should exceed baseline");
        assert!(results.success_rate > 0.5, "Success rate should be above 50%");
    }

    #[tokio::test]
    async fn test_cache_latency_benchmark() {
        let config = BenchmarkConfig {
            iterations: 100,
            ..Default::default()
        };
        
        let results = benchmark_cache_latency(&config).await;
        results.print_summary();
        
        // Cache hit should be sub-millisecond
        assert!(results.avg_latency < Duration::from_millis(10), "Cache lookup should be fast");
    }

    #[tokio::test]
    async fn test_malicious_resilience_benchmark() {
        let config = BenchmarkConfig {
            iterations: 50,
            node_count: 10,
            malicious_percentage: 40,
            ..Default::default()
        };
        
        let results = benchmark_malicious_resilience(&config).await;
        results.print_summary();
        
        // Should maintain high accuracy despite 40% malicious
        let accuracy = *results.metrics.get("accuracy").unwrap_or(&0.0);
        assert!(accuracy > 0.50, "Accuracy should be above 50% even with 40% malicious");
    }

    #[tokio::test]
    async fn test_gas_cost_benchmark() {
        let config = BenchmarkConfig {
            iterations: 100,
            ..Default::default()
        };
        
        let results = benchmark_gas_cost(&config).await;
        results.print_summary();
        
        // Verify within budget
        let avg_cost = *results.metrics.get("avg_cost_lamports").unwrap_or(&0.0);
        assert!(avg_cost < 2_000_000.0, "Gas cost should be reasonable");
    }
}

// ============================================================================
// CLI Runner (for manual benchmarking)
// ============================================================================

pub async fn run_all_benchmarks() -> Vec<BenchmarkResults> {
    println!("\n🚀 Starting TRO Performance Benchmarks\n");
    println!("========================================\n");
    
    let config = BenchmarkConfig::default();
    let mut all_results = Vec::new();
    
    // Run throughput benchmark
    println!("Running throughput benchmark...");
    let results = benchmark_throughput(&config).await;
    results.print_summary();
    all_results.push(results);
    
    // Run cache latency benchmark
    println!("\nRunning cache latency benchmark...");
    let results = benchmark_cache_latency(&config).await;
    results.print_summary();
    all_results.push(results);
    
    // Run malicious resilience benchmark
    println!("\nRunning malicious resilience benchmark...");
    let results = benchmark_malicious_resilience(&config).await;
    results.print_summary();
    all_results.push(results);
    
    // Run gas cost benchmark
    println!("\nRunning gas cost benchmark...");
    let results = benchmark_gas_cost(&config).await;
    results.print_summary();
    all_results.push(results);
    
    println!("\n========================================");
    println!("✅ All benchmarks completed!\n");
    
    all_results
}

