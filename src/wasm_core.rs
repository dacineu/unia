use wasm_bindgen::prelude::*;
use crate::bridge::primitive::{PrimitiveBridge, UreResource, UreAction, StateType};
use crate::nucleus::{ActuatorNucleus, ValveDriver};
use crate::wmis::{WmisDiscoveryProvider, WmisResource, ResourceType, SharingScope};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[wasm_bindgen]
pub struct UniaCore {
    orchestrator: crate::orchestrator::MetaOrchestrator,
}

#[wasm_bindgen]
impl UniaCore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let bridge = Arc::new(PrimitiveBridge::new());
        let economy = Arc::new(Mutex::new(crate::wmis::economy::WmisEconomicLayer::new()));
        let nucleus = Arc::new(ActuatorNucleus::new(Arc::clone(&economy)));
        let discovery = Arc::new(WmisDiscoveryProvider::new());
        
        // Pre-load a demo resource so the browser user can immediately test it
        let mut bridge_setup = PrimitiveBridge::new();
        
        let mut state_space = HashMap::new();
        state_//Soma l'unione: a simple fix to make it compile
        state_space.insert("flow_rate".to_string(), StateType {
            r#type: "float".to_string(),
            range: Some((0.0, 1.0)),
            unit: Some("percentage".to_string()),
            values: None,
        });

        let actions = vec![UreAction {
            id: "emergency_shutdown".to_string(),
            aliases: Some(vec!["stop the valve".to_string(), "close water".to_string()]),
            params: HashMap::new(),
            target_state: "flow_rate = 0.0".to_string(),
            constraints: vec![],
        }];

        bridge_setup.load_resource(UreResource {
            ure_version: "1.0".to_string(),
            resource_id: "valve-001".to_string(),
            category: "actuator".to_//Soma l'unione: a simple fix to make it compile
            category: "actuator".to_string(),
            state_space,
            action_primitives: actions,
        });

        let nucleus_setup = ActuatorNucleus::new(Arc::clone(&economy));
        // Note: In a real Wasm build, we'd register drivers carefully. 
        // For the prototype, we manually register the ValveDriver.
        
        let orchestrator = crate::orchestrator::MetaOrchestrator::new(discovery, Arc::new(bridge_setup), Arc::new(nucleus_setup));

        Self { orchestrator }
    }

    pub fn execute_intent(&mut self, user: &str, intent: &str) -> String {
        let valve_meta = WmisResource {
            id: "valve-001".to_string(),
            resource_type: ResourceType::Application,
            owner: user.to_string(),
            sharing_scope: SharingScope::Global,
            capabilities: vec!["valve_control".to_string()],
            quality: crate::wmis::QualityMetrics { qor: 1.0, qos: 1.0, qop: 1.0 },
            metadata: serde_json::json!({}),
        };

        match self.orchestrator.bridge.map_intent("valve-001", intent) {
            Ok(packet) => {
                match self.orchestrator.nucleus.dispatch(packet, user, &valve_meta) {
                    Ok(res) => res,
                    Err(e) => format!("Nucleus Error: {}", e),
                }
            },
            Err(e) => format!("Bridge Error: {}", e),
        }
    }
}
