use crate::identifiers::DuUuid;
use uuid::Uuid;
use serde_json::{Value, json};
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum MutationStatus {
    Stable,
    Evolving,
    Degraded,
    Collapsed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MutationType {
    Evolution,
    Involution,
    Correction,
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

    pub fn check_for_overflow(&self, current_performance: f64, baseline: f64) -> bool {
        current_performance < self.baseline_performance_threshold || current_performance < (baseline * 0.8)
    }

    pub fn mutate(&self, parent_id: Uuid, current_depth: u32, manifest: &Value, m_type: MutationType) -> Result<(Uuid, Value), Box<dyn Error>> {
        if current_depth >= self.mutation_depth_limit {
            return Err("Mutation depth limit reached. Resource is saturated.".into());
        }

        println!("Mutating resource {} via {:?}... Depth: {}", parent_id, m_type, current_depth);

        let mut new_manifest = manifest.clone();

        if let Some(guidance) = new_manifest["guidance"].as_str() {
            let evolved_guidance = format!("{} [MUTATION: {:?}, ITER: {}]", guidance, m_type, current_depth + 1);
            new_manifest["guidance"] = Value::String(evolved_guidance);
        }

        let mut provenance = new_manifest["provenance"].as_object().cloned().unwrap_or_else(|| serde_json::Map::new());
        provenance.insert("parent_uuid".to_string(), json!(parent_id.to_string()));
        provenance.insert("mutation_type".to_string(), json!(format!("{:?}", m_type)));
        new_manifest["provenance"] = Value::Object(provenance);

        let new_id = DuUuid::generate(&new_manifest, None)?;

        Ok((new_id, new_manifest))
    }

    pub fn prune_fat(&self, actuators: Vec<Uuid>, champion_id: Uuid) -> Vec<Uuid> {
        println!("Trimming the fat... Preserving Champion: {}", champion_id);
        actuators.into_iter()
            .filter(|id| *id == champion_id)
            .collect()
    }

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

        let (new_id, new_manifest) = engine.mutate(id, 0, &manifest, MutationType::Evolution).unwrap();
        assert_ne!(id, new_id);
        assert!(new_manifest["guidance"].as_str().unwrap().contains("MUTATION"));

        let result = engine.mutate(new_id, 3, &new_manifest, MutationType::Evolution);
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
