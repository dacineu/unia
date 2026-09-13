use crate::identifiers::DuUuid;
use crate::orchestrator::{BehavioralVector, ActivationVector};
use uuid::Uuid;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HybridActuator {
    pub source_ids: Vec<Uuid>,
    pub synthesized_prompt: String,
    pub merged_budget: usize,
    pub confidence_score: f64,
}

pub struct RouterSlm {
    // In production, this would be a call to a lightweight LLM (e.g., Phi-3, Llama-3-8B)
    // that specializes in resource blending.
    model_id: String,
}

impl RouterSlm {
    pub fn new(model_id: &str) -> Self {
        Self {
            model_id: model_id.to_string(),
        }
    }

    /// Synthesizes multiple "Floating" actuators into a single Hybrid Actuator.
    /// This is the "Fluid Factory" core: blending signals from the mesh.
    pub fn synthesize(&self, actuators: Vec<(Uuid, Value)>, vector: BehavioralVector) -> Result<HybridActuator, Box<dyn std::error::Error>> {
        if actuators.is_empty() {
            return Err("No actuators provided for synthesis".into());
        }

        // 1. Extract Guidance from all floating actuators
        let mut guidance_parts = Vec::new();
        let mut total_budget = 0;

        for (id, manifest) in &actuators {
            let guidance = manifest["guidance"].as_str().unwrap_or("");
            guidance_parts.push(format!("[Resource {}]: {}", id, guidance));
            total_budget += manifest["tokens_per_task"].as_u64().unwrap_or(1000) as usize;
        }

        // 2. Synthesis Logic (Simulated Router-SLM behavior)
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
    fn test_synthesis_blending() {
        let router = RouterSlm::new("phi-3-router");
        
        let actuators = vec![
            (Uuid::new_v4(), json!({"guidance": "Expert in Crypto-Liquidity", "tokens_per_task": 1000})),
            (Uuid::new_v4(), json!({"guidance": "Expert in Fiat-Gateways", "tokens_per_task": 1200})),
        ];

        let hybrid = router.synthesize(actuators, BehavioralVector::Smartest).unwrap();

        assert!(hybrid.synthesized_prompt.contains("SYNTHESIS_MODE: DEEP"));
        assert!(hybrid.synthesized_prompt.contains("Crypto-Liquidity"));
        assert!(hybrid.synthesized_prompt.contains("Fiat-Gateways"));
        assert!(hybrid.confidence_score > 0.0);
    }
}
