from pydantic_settings import BaseSettings
from typing import List

class Settings(BaseSettings):
    # Solana配置
    SOLANA_NETWORK: str = "devnet"
    SOLANA_RPC_URL: str = "https://api.devnet.solana.com"
    PROGRAM_ID: str = ""
    
    # IPFS配置
    IPFS_API_URL: str = "http://localhost:5001"
    PINATA_API_KEY: str = ""
    PINATA_SECRET_KEY: str = ""
    PINATA_GATEWAY_URL: str = "https://gateway.pinata.cloud/ipfs/"
    
    # 数据库配置
    DATABASE_URL: str = "postgresql://user:password@localhost:5432/daollm"
    REDIS_URL: str = "redis://localhost:6379"
    
    # 后端配置
    API_PORT: int = 8000
    API_HOST: str = "0.0.0.0"
    CORS_ORIGINS: List[str] = ["http://localhost:3000"]
    
    # LLM配置
    LOCAL_LLM_URL: str = "http://localhost:8001"
    LLM_MODEL: str = "llama3"
    INFERENCE_NODES: int = 3
    
    # 日志级别
    LOG_LEVEL: str = "INFO"
    
    class Config:
        env_file = ".env"
        case_sensitive = True

settings = Settings()

