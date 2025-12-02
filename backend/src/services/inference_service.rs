use anyhow::Result;
use serde_json::Value;

use crate::models::{InferenceResponse, RiskAssessment, NodeResult};

pub struct InferenceService {
    local_llm_url: String,
    inference_nodes: u32,
}

impl InferenceService {
    pub fn new() -> Self {
        let local_llm_url = std::env::var("LOCAL_LLM_URL")
            .unwrap_or_else(|_| "http://localhost:8001".to_string());
        let inference_nodes = std::env::var("INFERENCE_NODES")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);
        
        Self {
            local_llm_url,
            inference_nodes,
        }
    }
    
    pub async fn analyze_proposal(&self, proposal_id: &str, proposal_text: &str) -> Result<InferenceResponse> {
        // 模拟多个节点推理
        let mut node_results = Vec::new();
        
        for i in 0..self.inference_nodes {
            let result = self.simulate_node_inference(i, proposal_text).await?;
            node_results.push(result);
        }
        
        // 聚合结果（简单多数投票）
        let summary = self.aggregate_summaries(&node_results);
        let risk_assessment = self.aggregate_risks(&node_results);
        let recommendation = self.aggregate_recommendations(&node_results);
        let confidence = self.calculate_confidence(&node_results);
        
        Ok(InferenceResponse {
            proposal_id: proposal_id.to_string(),
            summary,
            risk_assessment,
            recommendation,
            confidence,
            node_results,
        })
    }
    
    async fn simulate_node_inference(&self, node_id: u32, proposal_text: &str) -> Result<NodeResult> {
        // TODO: 实际调用LLM API
        // 简化版本：返回模拟结果
        Ok(NodeResult {
            node_id: format!("node-{}", node_id),
            summary: format!("模拟摘要 from node-{}", node_id),
            confidence: 85,
        })
    }
    
    fn aggregate_summaries(&self, results: &[NodeResult]) -> String {
        // 简单聚合：取第一个节点的摘要
        results.first()
            .map(|r| r.summary.clone())
            .unwrap_or_else(|| "无法生成摘要".to_string())
    }
    
    fn aggregate_risks(&self, _results: &[NodeResult]) -> RiskAssessment {
        // 简化版本：返回中等风险
        RiskAssessment {
            technical: 5,
            financial: 6,
            governance: 4,
            execution: 3,
        }
    }
    
    fn aggregate_recommendations(&self, _results: &[NodeResult]) -> String {
        "建议支持，但需要进一步评估".to_string()
    }
    
    fn calculate_confidence(&self, results: &[NodeResult]) -> f64 {
        if results.is_empty() {
            return 0.0;
        }
        
        let sum: u32 = results.iter().map(|r| r.confidence as u32).sum();
        (sum as f64) / (results.len() as f64)
    }
    
    pub async fn get_nodes(&self) -> Result<Vec<Value>> {
        // TODO: 从Solana链上查询节点
        // 简化版本：返回模拟节点列表
        Ok(vec![
            serde_json::json!({
                "id": "node-0",
                "owner": "11111111111111111111111111111111",
                "stake_amount": 1000,
                "reputation_score": 85,
                "is_active": true
            })
        ])
    }
}

