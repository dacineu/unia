use crate::registry::ActuatorRegistry;
use crate::router::RouterSlm;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use sha2::{Sha256, Digest};

pub struct ExplorerHarvester {
    _registry: ActuatorRegistry,
    _router: RouterSlm,
}

impl ExplorerHarvester {
    pub fn new(_registry: ActuatorRegistry, _router: RouterSlm) -> Self {
        Self { _registry, _router }
    }

    /// Scans a directory for "Capability Seeds" (scripts, config files, docs)
    /// and attempts to convert them into Actuators.
    pub fn harvest_local_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
        let mut discovered_actuators = Vec::new();
        let root = path.as_ref();

        let entries = fs::read_dir(root)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["sh", "py", "md", "txt", "json"].contains(&ext_str.as_str()) {
                        if let Some(id) = self.analyze_and_actuate(&path)? {
                            discovered_actuators.push(id);
                        }
                    }
                }
            }
        }

        Ok(discovered_actuators)
    }

    /// Analyzes a file to extract guidance and complexity, then wraps it as a .ure.
    fn analyze_and_actuate(&self, path: &Path) -> Result<Option<Uuid>, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        
        // 1. Use the Router-SLM to extract "Guidance" and "Complexity" from the raw file.
        // In production, the Router-SLM would parse the code/text to summarize its a-priori capability.
        let extracted_guidance = format!("Extracted capability from: {}", path.display());
        let extracted_complexity = 0.5; 

        // 2. Generate DU-UUID based on the content
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x40; 
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let resource_id = Uuid::from_bytes(bytes);

        // 3. Wrap in a .ure manifest
        let manifest = serde_json::json!({
            "resource_id": resource_id.to_string(),
            "resource_type": "skill",
            "complexity_score": extracted_complexity,
            "guidance": extracted_guidance,
            "compressed_data": "base64_simulated_compressed_content",
            "source_path": path.to_string_lossy().to_string()
        });

        // 4. Register in the Mesh
        let temp_ure_path = format!("{}.ure", resource_id);
        fs::write(&temp_ure_path, serde_json::to_string(&manifest)?)?;
        
        // We use the registry to officially add it to the la-piece-de-résistance mesh
        // (Note: we assume register_ure_file is available in registry)
        // self.registry.register_ure_file(&temp_ure_path, None)?;
        
        let _ = fs::remove_file(temp_ure_path);

        Ok(Some(resource_id))
    }
}
