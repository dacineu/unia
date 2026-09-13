use crate::orchestrator::{BehavioralVector, ActivationVector};
use uuid::Uuid;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct HybridActuator {
    pub source_ids: Vec<Uuid>,
    pub synthesized_prompt: String,
    pub merged_budget: usize,
    pub confidence_score: f64,
}

impl HybridActuator {
    /// Converts a Hybrid Actuator into a standard ActivationVector for the SLM.
    pub fn to_activation_vector(&self, vector: BehavioralVector) -> ActivationVector {
        let is_deep_dive = vector == BehavioralVector::Smartest;
        
        ActivationVector {
            system_prompt: self.synthesized_prompt.clone(),
            token_budget: self.merged_budget,
            is_deep_dive,
            behavioral_mode: vector,
        }
    }
}

pub struct RouterSlm {
    _model_id: String,
}

impl RouterSlm {
    pub fn new(model_id: &str) -> Self {
        Self {
            _model_id: model_id.to_string(),
        }
    }

    pub fn synthesize(&self, actuators: Vec<(Uuid, Value)>, vector: BehavioralVector) -> Result<HybridActuator, Box<dyn std::error::Error>> {
        if actuators.is_empty() {
            return Err("No actuators provided for synthesis".into());
        }

        let mut guidance_parts = Vec::new();
        let mut total_budget = 0;

        for (id, manifest) in &actuators {
            let guidance = manifest["guidance"].as_str().unwrap_or("");
            guidance_parts.push(format!("[Resource {}]: {}", id, guidance));
            total_budget += manifest["tokens_per_task"].as_u64().unwrap_or(1000) as usize;
        }

        let blended_guidance = guidance_parts.join("\n");
        let synthesis_prompt = match vector {
            BehavioralVector::Quickest => format!(
                "SYNTHESIS_MODE: FAST\nCombine the following resources into the shortest possible execution path:\n{}\nResult: Only the essential sequence.",
                blended_guidance
            ),
            BehavioralVector::Smartest => format!(
                "SYNTHESIS_MODE: DEEP\nSynthesize a high-fidelity, multi-stage strategy by blending these resources:\n{}\nResult: A comprehensive, validated workflow.",
                blended_guidance
            ),
            BehavioralVector::Direct => format!(
                "SYNTHESIS_MODE: RAW\nMap these resources directly to a technical execution plan:\n{}\nResult: Technical specifications only.",
                blended_guidance
            ),
        };

        let confidence = (1.0 - (actuators.len() as f64 * 0.05)).max(0.5);

        Ok(HybridActuator {
            source_ids: actuators.iter().map(|(id, _)| *id).collect(),
            synthesized_prompt: synthesis_prompt,
            merged_budget: (total_budget / 2).max(1024),
            confidence_score: confidence,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_synthesis_to_activation() {
        let router = RouterSlm::new("phi-3-router");
        let actuators = vec![
            (Uuid::new_v4(), json!({"guidance": "A", "tokens_per_task": 1000})),
            (Uuid::new_v4(), json!({"guidance": "B", "tokens_per_task": 1000})),
        ];
        let hybrid = router.synthesize(actuators, BehavioralVector::Smartest).unwrap();
        let activation = hybrid.to_activation_vector(BehavioralVector::Smartest);
        
        assert!(activation.is_deep_dive);
        assert_eq!(activation.behavioral_mode, BehavioralVector::Smartest);
    }
}
