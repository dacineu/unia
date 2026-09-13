use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// The Universal Primitive IDs defined in the Primitive Library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UniversalPrimitive {
    // State-Query
    GetState,
    GetValue,
    CheckSense,
    // State-Transition
    SetValue,
    Toggle,
    Increment,
    Reset,
    // Flow & Signal
    Route,
    Pipe,
    Broadcast,
    // Temporal & Event
    Delay,
    Watch,
    Pulse,
    // Compute & Logic
    Compare,
    Transform,
    Validate,
}

/// The standard packet sent from the Bridge to the Actuator Nucleus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitivePacket {
    pub header: PacketHeader,
    pub payload: PacketPayload,
    pub context: PacketContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketHeader {
    pub timestamp: u64,
    pub request_id: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketPayload {
    pub primitive: UniversalPrimitive,
    pub resource_id: String,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketContext {
    pub expected_state: Option<String>,
    pub timeout_ms: u32,
}

/// Formal representation of a .ure resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UreResource {
    pub ure_version: String,
    pub resource_id: String,
    pub category: String,
    pub state_space: HashMap<String, StateType>,
    pub action_primitives: Vec<UreAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateType {
    pub r#type: String,
    pub range: Option<(f64, f64)>,
    pub unit: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UreAction {
    pub id: String,
    pub params: HashMap<String, String>,
    pub target_state: String,
    pub constraints: Vec<String>,
}

pub struct PrimitiveBridge {
    resources: HashMap<String, UreResource>,
}

impl PrimitiveBridge {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn load_resource(&mut self, resource: UreResource) {
        self.resources.insert(resource.resource_id.clone(), resource);
    }

    /// Maps natural language intent to a Universal Primitive Packet
    /// In a real scenario, this would use a small SLM or keyword mapping.
    pub fn map_intent(&self, resource_id: &str, intent: &str) -> Result<PrimitivePacket, String> {
        let resource = self.resources.get(resource_id)
            .ok_or_else(|| format!("Resource {} not found", resource_id))?;

        // 1. Match intent to a URE action
        // Improved matching: check if action ID (with underscores replaced by spaces) is in intent
        let action = resource.action_primitives.iter()
            .find(|a| {
                let normalized_id = a.id.to_lowercase().replace('_', " ");
                intent.to_lowercase().contains(&normalized_id)
            })
            .ok_or_else(|| format!("No matching action for intent '{}' in resource {}", intent, resource_id))?;

        // 2. Determine which Universal Primitive this action maps to
        // (Simplified mapping logic for the prototype)
        let primitive = self.resolve_primitive(&action.id, &action.target_state);

        // 3. Basic Constraint Check (Simulated)
        for constraint in &action.constraints {
            println!("[Bridge] Validating constraint: {}", constraint);
            // In a real system, we would check the actual current state of the resource
        }

        Ok(PrimitivePacket {
            header: PacketHeader {
                timestamp: 1694430000, // Simulated
                request_id: "req-abc-123".to_string(),
                priority: Priority::Medium,
            },
            payload: PacketPayload {
                primitive,
                resource_id: resource_id.to_string(),
                arguments: action.params.clone(),
            },
            context: PacketContext {
                expected_state: Some(action.target_state.clone()),
                timeout_ms: 500,
            },
        })
    }

    fn resolve_primitive(&self, action_id: &str, target_state: &str) -> UniversalPrimitive {
        if action_id.contains("shutdown") || action_id.contains("off") {
            UniversalPrimitive::Reset
        } else if action_id.contains("adjust") || action_id.contains("set") {
            UniversalPrimitive::SetValue
        } else if action_id.contains("check") || action_id.contains("get") {
            UniversalPrimitive::GetValue
        } else {
            UniversalPrimitive::SetValue // Default
        }
    }
}
