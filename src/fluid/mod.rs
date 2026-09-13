use uuid::Uuid;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::registry::ActuatorRegistry;
use crate::router::RouterSlm;

pub mod factory;
pub use factory::{FluidFactory, SynthesisRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotType {
    Logic,      // The core reasoning/execution logic
    Presenter,  // The UI/Output format (The la-piece-de-résistance interface)
    Auditor,    // The verification/security layer
    Optimizer,  // The efficiency/compression layer
}

pub struct MicroNucleus {
    pub id: Uuid,
    pub slots: HashMap<SlotType, Uuid>,
    pub context_state: Value,
}

pub struct FluidStateProjector {
    registry: Arc<ActuatorRegistry>,
    router: Arc<RouterSlm>,
    active_nucleus: Arc<RwLock<MicroNucleus>>,
}

impl FluidStateProjector {
    pub fn new(registry: Arc<ActuatorRegistry>, router: Arc<RouterSlm>, initial_nucleus: MicroNucleus) -> Self {
        Self {
            registry,
            router,
            active_nucleus: Arc::new(RwLock::new(initial_nucleus)),
        }
    }

    /// The "Flux Point": Hot-swaps a specific actuator without stopping the nucleus.
    pub fn flux_swap(&self, slot: SlotType, new_actuator_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        println!("🌊 Flux Point Triggered: Swapping {:?} to {}", slot, new_actuator_id);
        
        // Verify the new actuator exists in the registry
        self.registry.resolve_actuator(new_actuator_id)?;

        let mut nucleus = self.active_nucleus.write().unwrap();
        nucleus.slots.insert(slot, new_actuator_id);
        
        println!("✅ Flux complete. State projected to new variant.");
        Ok(())
    }

    /// Executes the current state of the micro-nucleus.
    pub fn execute(&self, input: &str) -> String {
        let nucleus = self.active_nucleus.read().unwrap();
        let logic_id = nucleus.slots.get(&SlotType::Logic).expect("Logic slot missing");
        let presenter_id = nucleus.slots.get(&SlotType::Presenter).expect("Presenter slot missing");

        // In a real system, we would route the input through Logic -> Auditor -> Presenter
        let logic_guidance = self.registry.resolve_actuator(*logic_id)
            .map(|m| m["guidance"].as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "Default Logic".to_string());

        let presenter_guidance = self.registry.resolve_actuator(*presenter_id)
            .map(|m| m["guidance"].as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "Default Presenter".to_string());

        format!("[STATE_PROJECTION]\nLogic: {}\nInterface: {}\nInput: {}", 
            logic_guidance, presenter_guidance, input)
    }

    pub fn get_current_id(&self) -> Uuid {
        self.active_nucleus.read().unwrap().id
    }
}
