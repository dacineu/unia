use crate::wmis::{WmisDiscoveryProvider, DiscoveryQuery, SharingScope, WmisResource};
use uuid::Uuid;
use std::collections::HashMap;
use crate::profiler::{Profiler, AgentProfile};
use crate::meta_actuators::MetaActuatorType;
use crate::bridge::primitive::{PrimitiveBridge, PrimitivePacket};
use crate::nucleus::ActuatorNucleus;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AgentSlot {
    pub id: Uuid,
    pub identity_profile: AgentProfile,
    pub actuator_type: MetaActuatorType,
    pub worktree_path: String,
    pub status: AgentStatus,
    pub linked_wmis_resource: Option<WmisResource>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Idle,
    Predicting,
    Processing,
    Completed,
    Failed,
}

pub struct MetaOrchestrator {
    slots: HashMap<Uuid, AgentSlot>,
    profiler: Profiler,
    discovery: Arc<WmisDiscoveryProvider>,
    // Integration with the la-piece-de-résistance pipeline
    pub bridge: Arc<PrimitiveBridge>,
    pub nucleus: Arc<ActuatorNucleus>,
}

impl MetaOrchestrator {
    pub fn new(discovery: Arc<WmisDiscoveryProvider>, bridge: Arc<PrimitiveBridge>, nucleus: Arc<ActuatorNucleus>) -> Self {
        Self {
            slots: HashMap::new(),
            profiler: Profiler::new(),
            discovery,
            bridge,
            nucleus,
        }
    }

    /// Predicts required resources for a given objective and pre-warms agent slots.
    pub fn predict_and_warm(&mut self, objective: &str) -> Vec<Uuid> {
        println!("🔮 Pre-emptively profiling objective: '{}'", objective);

        let predicted_capabilities = if objective.contains("rust") || objective.contains("code") {
            vec!["Rust Expert".to_string(), "Code Optimizer".to_string()]
        } else if objective.contains("data") || objective.contains("sql") {
            vec!["Data Architect".to_string(), "Query Optimizer".to_string()]
        } else {
            vec!["Generalist".to_string()]
        };

        let mut warmed_slots = Vec::new();

        for capability in predicted_capabilities {
            let query = DiscoveryQuery {
                seeker: "MetaOrchestrator".to_string(),
                allowed_scopes: vec![SharingScope::Global, SharingScope::Team],
                min_qor: 0.8,
                tags: vec![capability.clone()],
            };

            let found = self.discovery.discover_resources(query);

            let (role, spec, actuator_type) = if let Some(res) = found.first() {
                println!("✅ Found WMIS Champion for {}: {}", capability, res.id);
                ("Champion".to_string(), res.id.clone(), MetaActuatorType::Synthesizer)
            } else {
                println!("⚠️ No WMIS Champion for {}. Using synthetic profile.", capability);
                ("Synthetic".to_string(), capability.clone(), MetaActuatorType::Synthesizer)
            };

            let id = self.spawn_agent(&role, &spec, actuator_type);

            if let Some(slot) = self.slots.get_mut(&id) {
                slot.status = AgentStatus::Predicting;
                slot.linked_wmis_resource = found.first().cloned();
            }

            warmed_slots.push(id);
        }

        warmed_slots
    }

    /// Spawns a specialized agent with a pre-emptive identity.
    pub fn spawn_agent(&mut self, role: &str, specialization: &str, actuator_type: MetaActuatorType) -> Uuid {
        let id = Uuid::new_v4();

        let profile = AgentProfile {
            id,
            role: role.to_string(),
            specialization: specialization.to_string(),
            core_heuristics: vec!["Deterministic execution".to_string(), "Zero-redundancy logic".to_string()],
            optimization_goal: "Maximize la-piece-de-résistance precision".to_string(),
            capabilities: vec!["MCP Integration".to_string(), "DU-UUID Synthesis".to_string()],
            interaction_protocol: "Strict JSON-RPC / .ure manifest".to_string(),
        };

        let worktree_path = format!("/tmp/askillify/worktrees/{}", id);
        if let Err(e) = self.profiler.generate_identity_files(&worktree_path, &profile) {
            eprintln!("Failed to inject identity: {}", e);
        }

        println!("🚀 Spawned Agent {}: {} - {} (Worktree: {})", id, role, specialization, worktree_path);

        let slot = AgentSlot {
            id,
            identity_profile: profile,
            actuator_type,
            worktree_path,
            status: AgentStatus::Idle,
            linked_wmis_resource: None,
        };

        self.slots.insert(id, slot);
        id
    }

    /// Executes a la-piece-de-résistance pipeline: Intent -> Bridge -> Nucleus.
    pub fn execute_intent(&mut self, agent_id: &Uuid, intent: &str, user: &str) -> Result<String, String> {
        let slot = self.slots.get(agent_id)
            .ok_or_else(|| "Agent slot not found".to_string())?;

        let resource = slot.linked_wmis_resource.as_ref()
            .ok_or_else(|| "Agent has no linked WMIS resource for execution".to_string())?;

        println!("⚙️  Executing intent '{}' via Agent {}", intent, agent_id);

        // 1. Bridge Mapping (Intent -> Universal Primitive)
        let packet = self.bridge.map_intent(&resource.id, intent)?;

        // 2. Nucleus Dispatch (Primitive -> Hardware Action + Economic Charge)
        let result = self.nucleus.dispatch(packet, user, resource)?;

        Ok(result)
    }

    /// Fans one prompt across multiple specialized agents.
    pub fn fan_out(&mut self, prompt: &str) -> HashMap<Uuid, String> {
        println!("📡 Fanning out prompt: '{}' to fleet...", prompt);
        let mut results = HashMap::new();

        for (id, slot) in self.slots.iter_mut() {
            slot.status = AgentStatus::Processing;

            let resource_context = match &slot.linked_wmis_resource {
                Some(res) => format!(" using WMIS Champion {}", res.id),
                None => " using synthetic profile".to_string(),
            };

            let response = format!(
                "[{}] Response to '{}':- Processed via {} logic using identity {}{}.",
                slot.identity_profile.role,
                prompt,
                format!("{:?}", slot.actuator_type),
                slot.identity_profile.specialization,
                resource_context
            );

            results.insert(*id, response);
            slot.status = AgentStatus::Completed;
        }

        results
    }

    pub fn get_slot(&self, id: &Uuid) -> Option<&AgentSlot> {
        self.slots.get(id)
    }
}
