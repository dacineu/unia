use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::registry::ActuatorRegistry;
use crate::wmis::{WmisDiscoveryProvider, DiscoveryQuery, SharingScope};
use crate::fluid::{MicroNucleus, SlotType};

pub struct SynthesisRequest {
    pub objective: String,
    pub required_capabilities: Vec<String>,
    pub preferred_scope: SharingScope,
    pub min_qor: f64,
}

pub struct FluidFactory {
    registry: Arc<ActuatorRegistry>,
    discovery: Arc<WmisDiscoveryProvider>,
}

impl FluidFactory {
    pub fn new(registry: Arc<ActuatorRegistry>, discovery: Arc<WmisDiscoveryProvider>) -> Self {
        Self {
            registry,
            discovery,
        }
    }

    /// Synthesizes a new MicroNucleus by assembling the best matching actuators for the request.
    pub fn synthesize_nucleus(&self, request: SynthesisRequest) -> Result<MicroNucleus, Box<dyn std::error::Error>> {
        println!("🏭 Fluid Factory: Synthesizing Nucleus for objective: '{}'", request.objective);

        let mut slots = HashMap::new();

        // Mapping of SlotType to required capability keywords
        let slot_mapping = vec![
            (SlotType::Logic, "reasoning"),
            (SlotType::Presenter, "interface"),
            (SlotType::Auditor, "verification"),
            (SlotType::Optimizer, "efficiency"),
        ];

        for (slot, keyword) in slot_mapping {
            let actuator_id = self.resolve_best_actuator(keyword, &request)?;
            slots.insert(slot, actuator_id);
            println!("⚙️  Assigned {:?} slot -> {}", slot, actuator_id);
        }

        let nucleus_id = Uuid::new_v4();
        println!("✨ Nucleus synthesized successfully: {}", nucleus_id);

        Ok(MicroNucleus {
            id: nucleus_id,
            slots,
            context_state: serde_json::json!({
                "synthesis_objective": request.objective,
                "timestamp": "2026-09-13T00:00:00Z", // Simplified timestamp to avoid chrono dependency
            }),
        })
    }

    /// Resolves the best available actuator for a specific capability, 
    /// checking the local champion registry first, then the WMIS mesh.
    fn resolve_best_actuator(&self, capability: &str, request: &SynthesisRequest) -> Result<Uuid, Box<dyn std::error::Error>> {
        // 1. Check Local Champions
        if let Some(id) = self.registry.get_champion(capability) {
            println!("🏆 Local Champion found for {}: {}", capability, id);
            return Ok(id);
        }

        // 2. Search WMIS Mesh
        let query = DiscoveryQuery {
            seeker: "FluidFactory".to_string(),
            allowed_scopes: vec![request.preferred_scope.clone(), SharingScope::Global],
            min_qor: request.min_qor,
            tags: vec![capability.to_string()],
        };

        let found = self.discovery.discover_resources(query);
        if let Some(res) = found.first() {
            // In this prototype, we assume the WMIS ID can be parsed as a Uuid
            // In a real system, we'd use a mapping table or DU-UUID conversion
            if let Ok(id) = Uuid::parse_str(&res.id) {
                println!("🌐 WMIS Champion found for {}: {}", capability, id);
                return Ok(id);
            }
        }

        // 3. Fallback: Search local files for any matching .ure
        let matches = self.registry.find_matching_actuators(capability);
        if let Some((id, _)) = matches.first() {
            println!("🔍 Local Match found for {}: {}", capability, id);
            return Ok(*id);
        }

        Err(format!("Could not resolve any suitable actuator for capability: {}", capability).into())
    }
}
