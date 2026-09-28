use crate::registry::ActuatorRegistry;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

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

    pub fn collapse_actuator(
        &self,
        resource_id: Uuid,
        vector: BehavioralVector,
    ) -> Result<ActivationVector, Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    /// Collapse reads `complexity_score` off the manifest, so the fixture needs
    /// one. A shallow value is a surface skill; the vector decides the rest.
    fn orchestrator_for(dir: &std::path::Path) -> Orchestrator {
        Orchestrator::new(ActuatorRegistry::with_base_dir("mock", dir))
    }

    fn seeded(dir: &std::path::Path, score: f64) -> Uuid {
        let id = Uuid::new_v4();
        fs::write(
            dir.join(format!("{id}.ure")),
            serde_json::to_string(&json!({
                "complexity_score": score,
                "guidance": "Standard income generation path."
            }))
            .unwrap(),
        )
        .unwrap();
        id
    }

    #[test]
    fn the_quickest_vector_collapses_to_the_fast_actuator() {
        // Collapse is where a manifest becomes something an agent is told to
        // be. The two vectors must produce different instructions, or the
        // orchestrator is choosing nothing.
        let dir = tempfile::tempdir().unwrap();
        let orchestrator = orchestrator_for(dir.path());
        let id = seeded(dir.path(), 0.4);

        let activation = orchestrator
            .collapse_actuator(id, BehavioralVector::Quickest)
            .unwrap();

        assert!(
            !activation.is_deep_dive,
            "a shallow skill is not a deep dive"
        );
        assert!(
            activation.system_prompt.contains("[MODE:FAST]"),
            "got: {}",
            activation.system_prompt
        );
    }

    #[test]
    fn the_smartest_vector_collapses_to_the_deep_actuator() {
        // The same manifest under a different vector is the whole point of the
        // behavioural vector: one artifact, two manners of execution.
        let dir = tempfile::tempdir().unwrap();
        let orchestrator = orchestrator_for(dir.path());
        let id = seeded(dir.path(), 0.4);

        let activation = orchestrator
            .collapse_actuator(id, BehavioralVector::Smartest)
            .unwrap();

        assert!(activation.is_deep_dive, "Smartest is always a deep dive");
        assert!(
            activation.system_prompt.contains("[MODE:DEEP]"),
            "got: {}",
            activation.system_prompt
        );
    }

    #[test]
    fn a_manifest_deeper_than_the_threshold_collapses_to_deep_for_either_vector() {
        // Complexity overrides the vector. A genuinely deep artifact should not
        // be run shallow just because the caller asked for speed.
        let dir = tempfile::tempdir().unwrap();
        let orchestrator = orchestrator_for(dir.path());
        let id = seeded(dir.path(), 0.9);

        let activation = orchestrator
            .collapse_actuator(id, BehavioralVector::Quickest)
            .unwrap();

        assert!(activation.is_deep_dive);
    }

    #[test]
    fn collapsing_a_missing_manifest_is_an_error_rather_than_a_default() {
        // An absent manifest must not silently produce a usable-looking
        // activation, or a deleted artifact looks like a shallow skill.
        let dir = tempfile::tempdir().unwrap();
        let orchestrator = orchestrator_for(dir.path());

        let result = orchestrator.collapse_actuator(Uuid::new_v4(), BehavioralVector::Quickest);

        assert!(result.is_err(), "a missing manifest resolved to something");
    }
}
