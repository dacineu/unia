use crate::registry::ActuatorRegistry;
use serde_json::Value;
use uuid::Uuid;
use std::collections::HashMap;

pub mod meta;
pub use meta::MetaOrchestrator;

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
    pub behavioral_mode: BehavioralVector,
}

pub struct Orchestrator {
    registry: ActuatorRegistry,
    warm_cache: HashMap<Uuid, Value>,
}

impl Orchestrator {
    pub fn new(registry: ActuatorRegistry) -> Self {
        Self {
            registry,
            warm_cache: HashMap::new(),
        }
    }

    pub fn collapse_actuator(&self, resource_id: Uuid, vector: BehavioralVector) -> Result<ActivationVector, Box<dyn std::error::Error>> {
        let manifest = if let Some(m) = self.warm_cache.get(&resource_id) {
            m.clone()
        } else {
            self.registry.resolve_actuator(resource_id)?
        };

        let complexity_score = manifest["complexity_score"].as_f64().unwrap_or(0.5);

        let is_deep_dive = complexity_score > 0.7 || vector == BehavioralVector::Smartest;

        let (prompt, budget) = match vector {
            BehavioralVector::Quickest => {
                let p = format!("[MODE:FAST]\nResource: {}\nGuidance: {}\nConstraint: Absolute conciseness. No reasoning.",
                    resource_id, manifest["guidance"].as_str().unwrap_or(""));
                (p, 128)
            }
            BehavioralVector::Smartest => {
                let p = format!("[MODE:DEEP]\nResource: {}\nGuidance: {}\nConstraint: Full Chain-of-Thought. Validate every step.",
                    resource_id, manifest["guidance"].as_str().unwrap_or(""));
                (p, 2048)
            }
            BehavioralVector::Direct => {
                let p = format!("[MODE:RAW]\nResource: {}\nGuidance: {}\nConstraint: Technical execution only. No conversational filler.",
                    resource_id, manifest["guidance"].as_str().unwrap_or(""));
                (p, 512)
            }
        };

        Ok(ActivationVector {
            system_prompt: prompt,
            token_budget: budget,
            is_deep_dive,
            behavioral_mode: vector,
        })
    }

    pub fn add_to_cache(&mut self, id: Uuid, manifest: Value) {
        self.warm_cache.insert(id, manifest);
    }
}
