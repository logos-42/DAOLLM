use anyhow::Result;
use serde_json::Value;
use reqwest::Client;

pub struct IPFSService {
    client: Client,
    ipfs_url: String,
    pinata_api_key: Option<String>,
    pinata_secret: Option<String>,
    pinata_gateway: String,
}

impl IPFSService {
    pub fn new() -> Self {
        let ipfs_url = std::env::var("IPFS_API_URL")
            .unwrap_or_else(|_| "http://localhost:5001".to_string());
        
        let pinata_api_key = std::env::var("PINATA_API_KEY").ok();
        let pinata_secret = std::env::var("PINATA_SECRET_KEY").ok();
        let pinata_gateway = std::env::var("PINATA_GATEWAY_URL")
            .unwrap_or_else(|_| "https://gateway.pinata.cloud/ipfs/".to_string());
        
        Self {
            client: Client::new(),
            ipfs_url,
            pinata_api_key,
            pinata_secret,
            pinata_gateway,
        }
    }
    
    pub async fn upload_json(&self, data: Value) -> Result<String> {
        // 优先使用Pinata（如果配置了）
        if let (Some(api_key), Some(secret)) = (&self.pinata_api_key, &self.pinata_secret) {
            return self.upload_to_pinata(data, api_key, secret).await;
        }
        
        // 否则使用本地IPFS节点
        self.upload_to_local_ipfs(data).await
    }
    
    async fn upload_to_pinata(&self, data: Value, api_key: &str, secret: &str) -> Result<String> {
        let json_str = serde_json::to_string(&data)?;
        
        let form = reqwest::multipart::Form::new()
            .text("pinataOptions", r#"{"cidVersion":1}"#)
            .text("pinataMetadata", r#"{"name":"proposal"}"#)
            .part("file", reqwest::multipart::Part::text(json_str)
                .file_name("proposal.json")
                .mime_str("application/json")?);
        
        let response = self.client
            .post("https://api.pinata.cloud/pinning/pinFileToIPFS")
            .header("pinata_api_key", api_key)
            .header("pinata_secret_api_key", secret)
            .multipart(form)
            .send()
            .await?;
        
        let result: Value = response.json().await?;
        Ok(result["IpfsHash"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
            .to_string())
    }
    
    async fn upload_to_local_ipfs(&self, data: Value) -> Result<String> {
        let json_str = serde_json::to_string(&data)?;
        
        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::text(json_str)
                .file_name("proposal.json")
                .mime_str("application/json")?);
        
        let response = self.client
            .post(format!("{}/api/v0/add", self.ipfs_url))
            .multipart(form)
            .send()
            .await?;
        
        let result: Value = response.json().await?;
        Ok(result["Hash"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response"))?
            .to_string())
    }
    
    pub async fn retrieve(&self, ipfs_hash: &str) -> Result<Value> {
        // 优先从Pinata网关获取
        if !self.pinata_gateway.is_empty() {
            let url = format!("{}{}", self.pinata_gateway, ipfs_hash);
            let response = self.client.get(&url).send().await?;
            let data: Value = response.json().await?;
            return Ok(data);
        }
        
        // 否则从本地IPFS节点获取
        let url = format!("{}/api/v0/cat?arg={}", self.ipfs_url, ipfs_hash);
        let response = self.client.post(&url).send().await?;
        let data: Value = response.json().await?;
        Ok(data)
    }
}

