use crate::identifiers::DuUuid;
use serde_json::Value;
use uuid::Uuid;
use std::path::Path;
use std::fs;
use std::collections::HashMap;

pub struct ActuatorRegistry {
    _db_connection_string: String,
    champions: HashMap<String, Uuid>,
}

impl ActuatorRegistry {
    pub fn new(conn_str: &str) -> Self {
        Self {
            _db_connection_string: conn_str.to_string(),
            champions: HashMap::new(),
        }
    }

    pub fn set_champion(&mut self, capability: &str, id: Uuid) {
        println!("🏆 Champion set for {}: {}", capability, id);
        self.champions.insert(capability.to_string(), id);
    }

    pub fn get_champion(&self, capability: &str) -> Option<Uuid> {
        self.champions.get(capability).cloned()
    }

    pub fn register_ure_file<P: AsRef<Path>>(&self, path: P, encryption_key: Option<&[u8; 32]>) -> Result<Uuid, Box<dyn std::error::Error>> {
        let content_str = fs::read_to_string(path)?;
        let manifest: Value = serde_json::from_str(&content_str)?;
        let resource_id = DuUuid::generate(&manifest, encryption_key)?;

        println!("Registering Actuator: {} | ID: {}", manifest["resource_type"], resource_id);
        Ok(resource_id)
    }

    pub fn resolve_actuator(&self, id: Uuid) -> Result<Value, Box<dyn std::error::Error>> {
        let path = format!("{}.ure", id);
        let content = fs::read_to_string(path)?;
        let manifest: Value = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Scans the registry for actuators that match a prompt's intent.
    pub fn find_matching_actuators(&self, prompt: &str) -> Vec<(Uuid, Value)> {
        println!("Scanning mesh for patterns matching: '{}'...", prompt);
        let mut matches = Vec::new();

        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("ure") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(manifest) = serde_json::from_str::<Value>(&content) {
                            let guidance = manifest["guidance"].as_str().unwrap_or("");
                            if guidance.to_lowercase().contains(&prompt.to_lowercase()) ||
                               prompt.to_lowercase().contains(&guidance.to_lowercase()) {
                                if let Some(id_str) = manifest["resource_id"].as_str() {
                                    if let Ok(id) = Uuid::parse_str(id_str) {
                                        matches.push((id, manifest));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        matches
    }
}
