use crate::identifiers::DuUuid;
use crate::registry::ActuatorRegistry;
use serde_json::Value;
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehavioralVector {
    Quickest,
    Smartest,
    Direct,
}

#[derive(Debug, Clone)]
pub struct ActivationVector {
    pub system_prompt: String,
    pub token_budget: usize,
    pub is_deep_dive: bool,
}

pub struct Orchestrator {
    registry: ActuatorRegistry,
    // Warm-cache for frequently used actuators
    warm_cache: HashMap<Uuid, Value>,
}

impl Orchestrator {
    pub fn new(registry: ActuatorRegistry) -> Self {
        Self {
            registry,
            warm_cache: HashMap::new(),
        }
    }

    /// The "Collapse" function: Turns a Resource ID + Behavioral Vector into an Activation Vector.
    pub fn collapse_actuator(&self, resource_id: Uuid, vector: BehavioralVector) -> Result<ActivationVector, Box<dyn std::error::Error>> {
        // 1. Retrieve the Actuator (from cache or registry)
        let manifest = if let Some(m) = self.warm_cache.get(&resource_id) {
            m.clone()
        } else {
            self.registry.resolve_actuator(resource_id)?
        };

        let complexity_score = manifest["complexity_score"].as_f64().unwrap_or(0.5);

        // 2. Determine "Dive" depth based on complexity and vector
        // Surface if score <= 0.7 AND vector is Quickest.
        // Deep if score > 0.7 OR vector is Smartest.
        let is_deep_dive = complexity_score > 0.7 || vector == BehavioralVector::Smartest;

        // 3. Generate the activation vector based on the behavior
        let (prompt, budget) = match vector {
            BehavioralVector::Quickest => {
                let p = format!("ACTUATOR_MODE: FAST\nCONTENT: {}\nInstruction: Solve immediately with minimal tokens.", manifest["guidance"].as_str().unwrap_or(""));
                (p, 256)
            }
            BehavioralVector::Smartest => {
                let p = format!("ACTUATOR_MODE: DEEP\nCONTENT: {}\nInstruction: Perform a multi-step reasoning chain to ensure maximum quality.", manifest["guidance"].as_str().unwrap_or(""));
                (p, 2048)
            }
            BehavioralVector::Direct => {
                let p = format!("ACTUATOR_MODE: RAW\nCONTENT: {}\nInstruction: Execute raw logic without guidance overhead.", manifest["guidance"].as_str().unwrap_or(""));
                (p, 512)
            }
        };

        Ok(ActivationVector {
            system_prompt: prompt,
            token_budget: budget,
            is_deep_dive,
        })
    }

    pub fn add_to_cache(&mut self, id: Uuid, manifest: Value) {
        self.warm_cache.insert(id, manifest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ActuatorRegistry;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_collapse_logic() {
        let dir = tempdir().unwrap();
        let id = Uuid::new_v4();
        let file_path = dir.path().join(format!("{}.ure", id));
        
        let manifest = json!({
            "complexity_score": 0.4, // Easy
            "guidance": "Standard income generation path."
        });
        
        fs::write(file_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        let registry = ActuatorRegistry::new("mock_conn");
        let mut orchestrator = Orchestrator::new(registry);
        
        // Test Quickest (should be Surface)
        let act_quick = orchestrator.collapse_actuator(id, BehavioralVector::Quickest).unwrap();
        assert!(!act_quick.is_deep_dive);
        assert!(act_quick.system_prompt.contains("ACTUATOR_MODE: FAST"));
        
        // Test Smartest (should be Deep)
        let act_smart = orchestrator.collapse_actuator(id, BehavioralVector::Smartest).unwrap();
        assert!(act_smart.is_deep_dive);
        assert!(act_smart.system_prompt.contains("ACTUATOR_MODE: DEEP"));
    }
}
