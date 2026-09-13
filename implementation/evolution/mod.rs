use crate::identifiers::DuUuid;
use uuid::Uuid;
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum MutationStatus {
    Stable,
    Evolving,
    Degraded,
    Collapsed,
}

pub struct MutationEngine {
    pub mutation_depth_limit: u32,
    pub baseline_performance_threshold: f64,
}

impl MutationEngine {
    pub fn new(depth_limit: u32, threshold: f64) -> Self {
        Self {
            mutation_depth_limit: depth_limit,
            baseline_performance_threshold: threshold,
        }
    }

    /// Evaluates if an actuator needs to be "Externalized" (mutated).
    /// Triggered by "Capacity Overflow" (e.g., low success rate, high latency).
    pub fn check_for_overflow(&self, current_performance: f64, baseline: f64) -> bool {
        current_performance < self.baseline_performance_threshold || current_performance < (baseline * 0.8)
    }

    /// Simulates the mutation of an actuator via a larger Agentic SLM.
    /// In production, this would involve fine-tuning a LoRA or updating the super-prompt.
    pub fn mutate(&self, resource_id: Uuid, current_depth: u32, manifest: &Value) -> Result<(Uuid, Value), Box<dyn Error>> {
        if current_depth >= self.mutation_depth_limit {
            return Err("Mutation depth limit reached. Resource is saturated.".into());
        }

        println!("Externalizing resource {} for mutation... Depth: {}", resource_id, current_depth);

        // 1. Simulate Agentic SLM rewriting the manifest
        let mut new_manifest = manifest.clone();
        
        // Update the guidance to be "evolved"
        if let Some(guidance) = new_manifest["guidance"].as_str() {
            let evolved_guidance = format!("{} [EVOLVED ITERATION {}]", guidance, current_depth + 1);
            new_manifest["guidance"] = Value::String(evolved_guidance);
        }

        // 2. Generate a NEW DU-UUID for the mutated resource
        let new_id = DuUuid::generate(&new_manifest, None)?;

        Ok((new_id, new_manifest))
    }

    /// Validates the mutated actuator against the parent's baseline to prevent degradation.
    pub fn validate_mutation(&self, parent_perf: f64, child_perf: f64) -> MutationStatus {
        if child_perf >= parent_perf {
            MutationStatus::Stable
        } else if child_perf > (parent_perf * 0.9) {
            MutationStatus::Evolving
        } else {
            MutationStatus::Degraded
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mutation_lifecycle() {
        let engine = MutationEngine::new(3, 0.7);
        let manifest = json!({
            "guidance": "Initial skill set",
            "complexity_score": 0.5
        });
        let id = Uuid::new_v4();

        assert!(engine.check_for_overflow(0.5, 0.8));

        let (new_id, new_manifest) = engine.mutate(id, 0, &manifest).unwrap();
        assert_ne!(id, new_id);
        assert!(new_manifest["guidance"].as_str().unwrap().contains("EVOLVED"));

        let result = engine.mutate(new_id, 3, &new_manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_status() {
        let engine = MutationEngine::new(3, 0.7);
        assert_eq!(engine.validate_mutation(0.8, 0.85), MutationStatus::Stable);
        assert_eq!(engine.validate_mutation(0.8, 0.75), MutationStatus::Evolving);
        assert_eq!(engine.validate_mutation(0.8, 0.5), MutationStatus::Degraded);
    }
}
