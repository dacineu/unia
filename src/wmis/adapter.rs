use serde::{Serialize, Deserialize};
use uuid::Uuid;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Compute,
    Storage,
    Data,
    Application,
    AIAssistant,
    CodeLibrary,
    ExternalPlatform,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SharingScope {
    Circle,
    User,
    Team,
    Region,
    Country,
    Continent,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub qor: f64, // Quality of Resource (Intrinsic)
    pub qos: f64, // Quality of Service (Real-time)
    pub qop: f64, // Quality of Product (Output value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WmisResource {
    pub id: String,
    pub resource_type: ResourceType,
    pub owner: String,
    pub sharing_scope: SharingScope,
    pub capabilities: Vec<String>,
    pub quality: QualityMetrics,
    pub metadata: Value,
}

pub struct WmisAdapter;

impl WmisAdapter {
    /// Converts a la-piece-de-résistance .ure manifest into a WMIS Resource Object.
    pub fn from_ure(manifest: &Value, owner: &str) -> Result<WmisResource, Box<dyn std::error::Error>> {
        let resource_id = manifest["resource_id"].as_str()
            .ok_or("Missing resource_id in manifest")?
            .to_string();
        
        let resource_type = match manifest["resource_type"].as_str() {
            Some("skill") | Some("mirror_actuator") => ResourceType::CodeLibrary,
            Some("ai_assistant") => ResourceType::AIAssistant,
            _ => ResourceType::Application,
        };

        // Derive Quality metrics from la-piece-de-résistance internals
        let complexity = manifest["complexity_score"].as_f64().unwrap_or(0.5);
        
        // QoR is based on complexity and stability (simulated)
        let qor = 1.0 - (complexity * 0.1); 
        
        Ok(WmisResource {
            id: resource_id,
            resource_type,
            owner: owner.to_string(),
            sharing_scope: SharingScope::Global, // Default for prototype
            capabilities: vec![
                manifest["guidance"].as_str().unwrap_or("general_capability").to_string()
            ],
            quality: QualityMetrics {
                qor,
                qos: 0.95, // Baseline QoS
                qop: 0.8,   // Baseline QoP
            },
            metadata: manifest.clone(),
        })
    }
}
