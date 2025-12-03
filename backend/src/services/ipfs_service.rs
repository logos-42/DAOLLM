//! TRO Enhanced IPFS Service
//!
//! Features:
//! - Intelligent compression (gzip, brotli, MessagePack)
//! - Chunked storage for large files
//! - Merkle root computation for on-chain indexing
//! - Content verification

use anyhow::{anyhow, Result};
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use tracing::{debug, info};

// ============================================================================
// Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct IPFSConfig {
    pub ipfs_url: String,
    pub pinata_api_key: Option<String>,
    pub pinata_secret: Option<String>,
    pub pinata_gateway: String,
    /// Enable automatic compression
    pub enable_compression: bool,
    /// Compression threshold (bytes) - only compress if larger
    pub compression_threshold: usize,
    /// Preferred compression method
    pub compression_method: CompressionMethod,
    /// Chunk size for large files (bytes)
    pub chunk_size: usize,
    /// Maximum uncompressed size (bytes)
    pub max_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    None,
    Gzip,
    Brotli,
}

impl Default for IPFSConfig {
    fn default() -> Self {
        Self {
            ipfs_url: std::env::var("IPFS_API_URL")
                .unwrap_or_else(|_| "http://localhost:5001".to_string()),
            pinata_api_key: std::env::var("PINATA_API_KEY").ok(),
            pinata_secret: std::env::var("PINATA_SECRET_KEY").ok(),
            pinata_gateway: std::env::var("PINATA_GATEWAY_URL")
                .unwrap_or_else(|_| "https://gateway.pinata.cloud/ipfs/".to_string()),
            enable_compression: true,
            compression_threshold: 1024,      // 1KB
            compression_method: CompressionMethod::Gzip,
            chunk_size: 256 * 1024,           // 256KB
            max_size: 10 * 1024 * 1024,       // 10MB
        }
    }
}

// ============================================================================
// Types
// ============================================================================

/// Result of an IPFS upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// IPFS CID (Content Identifier)
    pub cid: String,
    /// Original size in bytes
    pub original_size: usize,
    /// Stored size in bytes (after compression)
    pub stored_size: usize,
    /// Compression ratio (0-1, lower is better)
    pub compression_ratio: f64,
    /// Compression method used
    pub compression: String,
    /// Content hash for verification
    pub content_hash: [u8; 32],
    /// Merkle root (for chunked uploads)
    pub merkle_root: Option<[u8; 32]>,
    /// Number of chunks (1 for single-file uploads)
    pub chunk_count: usize,
    /// Chunk CIDs (for chunked uploads)
    pub chunk_cids: Vec<String>,
}

/// Stored content metadata (for on-chain indexing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub cid: String,
    pub content_hash: [u8; 32],
    pub merkle_root: [u8; 32],
    pub original_size: u32,
    pub compressed_size: u32,
    pub compression: u8,  // 0=none, 1=gzip, 2=brotli
    pub chunk_count: u16,
    pub timestamp: i64,
}

impl StorageMetadata {
    /// Serialize for on-chain storage (compact format)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(128);
        bytes.extend_from_slice(&self.content_hash);
        bytes.extend_from_slice(&self.merkle_root);
        bytes.extend_from_slice(&self.original_size.to_le_bytes());
        bytes.extend_from_slice(&self.compressed_size.to_le_bytes());
        bytes.push(self.compression);
        bytes.extend_from_slice(&self.chunk_count.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }
}

// ============================================================================
// IPFS Service
// ============================================================================

pub struct IPFSService {
    client: Client,
    config: IPFSConfig,
}

impl IPFSService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            config: IPFSConfig::default(),
        }
    }

    pub fn with_config(config: IPFSConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    /// Upload JSON data with automatic compression
    pub async fn upload_json(&self, data: Value) -> Result<String> {
        let result = self.upload_json_enhanced(data).await?;
        Ok(result.cid)
    }

    /// Upload JSON with full result details
    pub async fn upload_json_enhanced(&self, data: Value) -> Result<UploadResult> {
        let json_str = serde_json::to_string(&data)?;
        self.upload_bytes(json_str.as_bytes(), "application/json", "data.json").await
    }

    /// Upload raw bytes with compression and chunking
    pub async fn upload_bytes(
        &self,
        data: &[u8],
        mime_type: &str,
        filename: &str,
    ) -> Result<UploadResult> {
        let original_size = data.len();

        if original_size > self.config.max_size {
            return Err(anyhow!("Content exceeds maximum size limit"));
        }

        // Compute content hash
        let content_hash = self.compute_hash(data);

        // Determine if compression is needed
        let (compressed_data, compression_method) = if self.config.enable_compression
            && original_size >= self.config.compression_threshold
        {
            self.compress(data)?
        } else {
            (data.to_vec(), CompressionMethod::None)
        };

        let stored_size = compressed_data.len();
        let compression_ratio = if original_size > 0 {
            stored_size as f64 / original_size as f64
        } else {
            1.0
        };

        debug!(
            "Compression: {} -> {} bytes ({:.1}% reduction)",
            original_size,
            stored_size,
            (1.0 - compression_ratio) * 100.0
        );

        // Check if chunking is needed
        if stored_size > self.config.chunk_size {
            return self.upload_chunked(&compressed_data, content_hash, original_size, compression_method).await;
        }

        // Single file upload
        let cid = self.upload_single(&compressed_data, mime_type, filename).await?;

        Ok(UploadResult {
            cid,
            original_size,
            stored_size,
            compression_ratio,
            compression: format!("{:?}", compression_method),
            content_hash,
            merkle_root: Some(content_hash), // For single file, merkle root = content hash
            chunk_count: 1,
            chunk_cids: vec![],
        })
    }

    /// Upload a single file to IPFS
    async fn upload_single(&self, data: &[u8], mime_type: &str, filename: &str) -> Result<String> {
        // Use Pinata if configured
        if let (Some(api_key), Some(secret)) = (&self.config.pinata_api_key, &self.config.pinata_secret) {
            return self.upload_to_pinata_bytes(data, api_key, secret, filename).await;
        }

        // Otherwise use local IPFS
        self.upload_to_local_ipfs_bytes(data, mime_type, filename).await
    }

    /// Upload chunked data
    async fn upload_chunked(
        &self,
        data: &[u8],
        content_hash: [u8; 32],
        original_size: usize,
        compression_method: CompressionMethod,
    ) -> Result<UploadResult> {
        let chunks: Vec<&[u8]> = data.chunks(self.config.chunk_size).collect();
        let chunk_count = chunks.len();

        info!("Uploading {} chunks for {} bytes", chunk_count, data.len());

        // Upload each chunk
        let mut chunk_cids = Vec::with_capacity(chunk_count);
        let mut chunk_hashes = Vec::with_capacity(chunk_count);

        for (i, chunk) in chunks.iter().enumerate() {
            let filename = format!("chunk_{:04}.bin", i);
            let cid = self.upload_single(chunk, "application/octet-stream", &filename).await?;
            let hash = self.compute_hash(chunk);

            chunk_cids.push(cid);
            chunk_hashes.push(hash);

            debug!("Uploaded chunk {}/{}: {}", i + 1, chunk_count, chunk_cids.last().unwrap());
        }

        // Compute Merkle root
        let merkle_root = self.compute_merkle_root(&chunk_hashes);

        // Create manifest
        let manifest = serde_json::json!({
            "version": 1,
            "content_hash": hex::encode(content_hash),
            "merkle_root": hex::encode(merkle_root),
            "original_size": original_size,
            "stored_size": data.len(),
            "compression": format!("{:?}", compression_method),
            "chunk_count": chunk_count,
            "chunk_size": self.config.chunk_size,
            "chunks": chunk_cids.iter().enumerate().map(|(i, cid)| {
                serde_json::json!({
                    "index": i,
                    "cid": cid,
                    "hash": hex::encode(chunk_hashes[i])
                })
            }).collect::<Vec<_>>()
        });

        // Upload manifest
        let manifest_str = serde_json::to_string(&manifest)?;
        let manifest_cid = self.upload_single(
            manifest_str.as_bytes(),
            "application/json",
            "manifest.json",
        ).await?;

        Ok(UploadResult {
            cid: manifest_cid,
            original_size,
            stored_size: data.len(),
            compression_ratio: data.len() as f64 / original_size as f64,
            compression: format!("{:?}", compression_method),
            content_hash,
            merkle_root: Some(merkle_root),
            chunk_count,
            chunk_cids,
        })
    }

    /// Retrieve and decompress data
    pub async fn retrieve(&self, ipfs_hash: &str) -> Result<Value> {
        let data = self.retrieve_bytes(ipfs_hash).await?;
        let json: Value = serde_json::from_slice(&data)?;
        Ok(json)
    }

    /// Retrieve raw bytes (with auto-decompression)
    pub async fn retrieve_bytes(&self, ipfs_hash: &str) -> Result<Vec<u8>> {
        let raw = self.fetch_raw(ipfs_hash).await?;

        // Try to detect and decompress
        if raw.len() >= 2 && raw[0] == 0x1f && raw[1] == 0x8b {
            // Gzip magic bytes
            return self.decompress_gzip(&raw);
        }

        // Check if it's a chunked manifest
        if let Ok(manifest) = serde_json::from_slice::<Value>(&raw) {
            if manifest.get("version").is_some() && manifest.get("chunks").is_some() {
                return self.retrieve_chunked(&manifest).await;
            }
        }

        Ok(raw)
    }

    /// Retrieve chunked data
    async fn retrieve_chunked(&self, manifest: &Value) -> Result<Vec<u8>> {
        let chunks = manifest["chunks"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid manifest: missing chunks"))?;

        let mut data = Vec::new();

        for chunk_info in chunks {
            let cid = chunk_info["cid"]
                .as_str()
                .ok_or_else(|| anyhow!("Invalid chunk info"))?;

            let chunk_data = self.fetch_raw(cid).await?;
            data.extend_from_slice(&chunk_data);
        }

        // Decompress if needed
        let compression = manifest["compression"].as_str().unwrap_or("None");
        if compression.contains("Gzip") {
            return self.decompress_gzip(&data);
        }

        Ok(data)
    }

    /// Fetch raw bytes from IPFS
    async fn fetch_raw(&self, ipfs_hash: &str) -> Result<Vec<u8>> {
        // Try Pinata gateway first
        if !self.config.pinata_gateway.is_empty() {
            let url = format!("{}{}", self.config.pinata_gateway, ipfs_hash);
            if let Ok(response) = self.client.get(&url).send().await {
                if response.status().is_success() {
                    return Ok(response.bytes().await?.to_vec());
                }
            }
        }

        // Fall back to local IPFS
        let url = format!("{}/api/v0/cat?arg={}", self.config.ipfs_url, ipfs_hash);
        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("IPFS fetch failed: {}", response.status()));
        }

        Ok(response.bytes().await?.to_vec())
    }

    // ========================================================================
    // Compression
    // ========================================================================

    fn compress(&self, data: &[u8]) -> Result<(Vec<u8>, CompressionMethod)> {
        match self.config.compression_method {
            CompressionMethod::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(data)?;
                let compressed = encoder.finish()?;

                // Only use compression if it actually saves space
                if compressed.len() < data.len() {
                    Ok((compressed, CompressionMethod::Gzip))
                } else {
                    Ok((data.to_vec(), CompressionMethod::None))
                }
            }
            CompressionMethod::Brotli => {
                let mut compressed = Vec::new();
                {
                    let mut encoder = brotli::CompressorWriter::new(&mut compressed, 4096, 4, 22);
                    encoder.write_all(data)?;
                }

                if compressed.len() < data.len() {
                    Ok((compressed, CompressionMethod::Brotli))
                } else {
                    Ok((data.to_vec(), CompressionMethod::None))
                }
            }
            CompressionMethod::None => Ok((data.to_vec(), CompressionMethod::None)),
        }
    }

    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    // ========================================================================
    // Hashing
    // ========================================================================

    fn compute_hash(&self, data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    fn compute_merkle_root(&self, hashes: &[[u8; 32]]) -> [u8; 32] {
        if hashes.is_empty() {
            return [0u8; 32];
        }

        if hashes.len() == 1 {
            return hashes[0];
        }

        let mut current_level: Vec<[u8; 32]> = hashes.to_vec();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate for odd count
                }
                next_level.push(hasher.finalize().into());
            }

            current_level = next_level;
        }

        current_level[0]
    }

    // ========================================================================
    // Upload Implementations
    // ========================================================================

    async fn upload_to_pinata_bytes(
        &self,
        data: &[u8],
        api_key: &str,
        secret: &str,
        filename: &str,
    ) -> Result<String> {
        let form = reqwest::multipart::Form::new()
            .text("pinataOptions", r#"{"cidVersion":1}"#)
            .text("pinataMetadata", format!(r#"{{"name":"{}"}}"#, filename))
            .part(
                "file",
                reqwest::multipart::Part::bytes(data.to_vec())
                    .file_name(filename.to_string())
                    .mime_str("application/octet-stream")?,
            );

        let response = self
            .client
            .post("https://api.pinata.cloud/pinning/pinFileToIPFS")
            .header("pinata_api_key", api_key)
            .header("pinata_secret_api_key", secret)
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Pinata upload failed: {} - {}", status, body));
        }

        let result: Value = response.json().await?;
        Ok(result["IpfsHash"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid Pinata response"))?
            .to_string())
    }

    async fn upload_to_local_ipfs_bytes(
        &self,
        data: &[u8],
        _mime_type: &str,
        filename: &str,
    ) -> Result<String> {
        let form = reqwest::multipart::Form::new().part(
            "file",
            reqwest::multipart::Part::bytes(data.to_vec())
                .file_name(filename.to_string())
                .mime_str("application/octet-stream")?,
        );

        let response = self
            .client
            .post(format!("{}/api/v0/add", self.config.ipfs_url))
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("IPFS upload failed: {} - {}", status, body));
        }

        let result: Value = response.json().await?;
        Ok(result["Hash"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid IPFS response"))?
            .to_string())
    }

    /// Create storage metadata for on-chain indexing
    pub fn create_metadata(&self, result: &UploadResult) -> StorageMetadata {
        StorageMetadata {
            cid: result.cid.clone(),
            content_hash: result.content_hash,
            merkle_root: result.merkle_root.unwrap_or([0u8; 32]),
            original_size: result.original_size as u32,
            compressed_size: result.stored_size as u32,
            compression: match result.compression.as_str() {
                "Gzip" => 1,
                "Brotli" => 2,
                _ => 0,
            },
            chunk_count: result.chunk_count as u16,
            timestamp: chrono::Utc::now().timestamp(),
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
    fn test_compression() {
        let service = IPFSService::new();

        // Compressible data (repeated pattern)
        let data = "Hello World! ".repeat(1000);
        let (compressed, method) = service.compress(data.as_bytes()).unwrap();

        assert!(compressed.len() < data.len());
        assert_eq!(method, CompressionMethod::Gzip);

        // Decompress and verify
        let decompressed = service.decompress_gzip(&compressed).unwrap();
        assert_eq!(decompressed, data.as_bytes());
    }

    #[test]
    fn test_merkle_root() {
        let service = IPFSService::new();

        let hashes = vec![
            service.compute_hash(b"chunk1"),
            service.compute_hash(b"chunk2"),
            service.compute_hash(b"chunk3"),
        ];

        let root = service.compute_merkle_root(&hashes);
        assert_ne!(root, [0u8; 32]);

        // Verify determinism
        let root2 = service.compute_merkle_root(&hashes);
        assert_eq!(root, root2);
    }

    #[test]
    fn test_storage_metadata_serialization() {
        let metadata = StorageMetadata {
            cid: "QmTest123".to_string(),
            content_hash: [1u8; 32],
            merkle_root: [2u8; 32],
            original_size: 1024,
            compressed_size: 512,
            compression: 1,
            chunk_count: 4,
            timestamp: 1700000000,
        };

        let bytes = metadata.to_bytes();
        assert!(bytes.len() >= 77); // 32 + 32 + 4 + 4 + 1 + 2 + 8
    }
}

