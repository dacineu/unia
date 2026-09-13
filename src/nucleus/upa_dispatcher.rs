use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::bridge::upa::{UpaOp, Dimensionality, UpaPacket};
use crate::nucleus::{ActuatorDriver};

/// The UPA Dispatcher manages "Computational Liquidity" by routing
/// architecture-agnostic UpaOps to the best available physical resource.
pub struct UpaDispatcher {
    /// Map of physical resource ID to its capabilities
    physical_resources: HashMap<String, Dimensionality>,
    /// Current primary processor for each virtual compute slot
    active_routes: HashMap<String, String>,
    /// State mirroring registry for hotswapping (Primary -> Shadow)
    shadow_routes: HashMap<String, String>,
}

impl UpaDispatcher {
    pub fn new() -> Self {
        Self {
            physical_resources: HashMap::new(),
            active_routes: HashMap::new(),
            shadow_routes: HashMap::new(),
        }
    }

    pub fn register_resource(&mut self, id: String, dim: Dimensionality) {
        self.physical_resources.insert(id, dim);
    }

    /// Sets the primary processor for a virtual compute slot
    pub fn set_primary(&mut self, slot: String, resource_id: String) {
        self.active_routes.insert(slot, resource_id);
    }

    /// Sets a shadow processor for mirroring state during hotswap
    pub fn set_shadow(&mut self, slot: String, resource_id: String) {
        self.shadow_routes.insert(slot, resource_id);
    }

    /// Routes a UpaPacket to the appropriate physical resource(s)
    pub fn route(&self, slot: &str, packet: &UpaPacket) -> Vec<String> {
        let mut targets = Vec::new();

        if let Some(primary) = self.active_routes.get(slot) {
            targets.push(primary.clone());
        }

        // If mirroring is active, also route to the shadow
        if let Some(shadow) = self.shadow_routes.get(slot) {
            targets.push(shadow.clone());
        }

        targets
    }

    pub fn clear_shadow(&mut self, slot: &str) {
        self.shadow_routes.remove(slot);
    }
}
