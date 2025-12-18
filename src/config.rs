use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfig {
    pub chain_name: String,      // Es: "LogisticaChain"
    pub version: String,         // Es: "1.0-Enterprise"
    pub db_path: String,         // Es: "./db_rossi"
    pub node_role: String,       // Es: "Master Node" o "Warehouse Node"
    pub server_port: u16,        // Porta P2P (es. 0 per automatica)
}

impl NodeConfig {
    // Funzione per caricare la configurazione da un file JSON
    pub fn load(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let config: NodeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}