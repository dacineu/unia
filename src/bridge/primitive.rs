use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
    pub aliases: Option<Vec<String>>,
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

    /// Dynamically load all .ure files from a directory
    pub fn load_resources_from_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let entries = fs::read_dir(path).map_err(|e| format!("Failed to read dir: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ure") {
                let content = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
                let resource: UreResource = serde_yaml::from_str(&content)
                    .map_err(|e| format!("YAML parse error in {:?}: {}", path, e))?;
                
                self.resources.insert(resource.resource_id.clone(), resource);
                println!("[Bridge] Loaded resource: {}", entry.file_name().to_string_lossy());
            }
        }
        Ok(())
    }

    pub fn load_resource(&mut self, resource: UreResource) {
        self.resources.insert(resource.resource_id.clone(), resource);
    }

    /// Maps natural language intent to a Universal Primitive Packet
    pub fn map_intent(&self, resource_id: &str, intent: &str) -> Result<PrimitivePacket, String> {
        let resource = self.resources.get(resource_id)
            .ok_or_else(|| format!("Resource {} not found", resource_id))?;

        let intent_lower = intent.to_lowercase();
        
        // 1. Exact/Containment Match (Fast Path)
        let mut best_action = None;
        let mut max_score = 0.0;

        for action in &resource.action_primitives {
            let normalized_id = action.id.to_lowercase().replace('_', " ");
            if intent_lower.contains(&normalized_id) {
                return self.create_packet(resource_id, action);
            }
            
            if let Some(aliases) = &action.aliases {
                for alias in aliases {
                    if intent_lower.contains(&alias.to_lowercase()) {
                        return self.create_packet(resource_id, action);
                    }
                }
            }

            // 2. Semantic Scoring (Slow Path)
            let score = crate::bridge::semantic::SemanticMapper::compute_score(&intent_lower, &action.id);
            if score > max_score {
                max_score = score;
                best_action = Some(action);
            }
        }

        // Threshold for semantic match (0.3 = moderate overlap)
        if max_score > 0.3 {
            println!("[Bridge] Semantic match found (score: {:.2})", max_score);
            return self.create_packet(resource_id, best_action.unwrap());
        }

        Err(format!("No matching action for intent '{}' in resource {}", intent, resource_id))
    }

    fn create_packet(&self, resource_id: &str, action: &UreAction) -> Result<PrimitivePacket, String> {
        let primitive = self.resolve_primitive(&action.id, &action.target_state);

        for constraint in &action.constraints {
            println!("[Bridge] Validating constraint: {}", constraint);
        }

        Ok(PrimitivePacket {
            header: PacketHeader {
                timestamp: 1694430000,
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

    fn resolve_primitive(&self, action_id: &str, _target_state: &str) -> UniversalPrimitive {
        if action_id.contains("shutdown") || action_id.contains("off") {
            UniversalPrimitive::Reset
        } else if action_id.contains("adjust") || action_id.contains("set") {
            UniversalPrimitive::SetValue
        } else if action_id.contains("check") || action_id.contains("get") {
            UniversalPrimitive::GetValue
        } else {
            UniversalPrimitive::SetValue
        }
    }
}
