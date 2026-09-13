use wasm_bindgen::prelude::*;
use std::sync::{Arc, Mutex};
use crate::bridge::primitive::{PrimitiveBridge, UreResource, UreAction, StateType};
use crate::nucleus::ActuatorNucleus;
use crate::wmis::{WmisDiscoveryProvider, WmisResource, ResourceType, SharingScope, QualityMetrics};
use std::collections::HashMap;

#[wasm_bindgen]
pub struct UniaCore {
    orchestrator: crate::orchestrator::MetaOrchestrator,
}

#[wasm_bindgen]
impl UniaCore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let economy = Arc::new(Mutex::new(crate::wmis::economy::WmisEconomicLayer::new()));
        let nucleus = Arc::new(ActuatorNucleus::new(Arc::clone(&economy)));
        let discovery = Arc::new(WmisDiscoveryProvider::new());
        
        let mut bridge_setup = PrimitiveBridge::new();

        // Pre-load a UPA Virtual Processor resource
        // This demonstrates "Computational Liquidity" in the browser
        let mut state_space = HashMap::new();
        state_space.insert("cpu_load".to_string(), StateType {
            r#type: "float".to_string(),
            range: Some((0.0, 100.0)),
            unit: Some("percent".to_string()),
            values: None,
        });

        let actions = vec![UreAction {
            id: "compute_sum".to_string(),
            aliases: Some(vec!["add".to_string(), "suma".to_string(), "sum".to_string()]),
            params: HashMap::new(),
            target_state: "result = sum(A, B)".to_string(),
            constraints: vec![],
        }];

        bridge_setup.load_resource(UreResource {
            ure_version: "1.0".to_string(),
            resource_id: "upa-virtual-01".to_string(),
            category: "processor".to_string(),
            state_space,
            action_primitives: actions,
        });

        let orchestrator = crate::orchestrator::MetaOrchestrator::new(
            discovery, 
            Arc::new(bridge_setup), 
            Arc::clone(&nucleus)
        );

        Self { orchestrator }
    }

    pub fn execute_intent(&mut self, user: &str, intent: &str) -> String {
        let resource_id = "upa-virtual-01";
        let upa_meta = WmisResource {
            id: resource_id.to_string(),
            resource_type: ResourceType::Virtual,
            owner: user.to_string(),
            sharing_scope: SharingScope::Global,
            capabilities: vec!["upa_compute".to_string()],
            quality: QualityMetrics { qor: 1.0, qos: 1.0, qop: 1.0 },
            metadata: serde_json::json!({}),
        };

        match self.orchestrator.bridge.map_intent(resource_id, intent) {
            Ok(packet) => {
                match self.orchestrator.nucleus.dispatch(packet, user, &upa_meta) {
                    Ok(res) => format!("[UPA Success] {}", res),
                    Err(e) => format!("[UPA Dispatch Error] {}", e),
                }
            },
            Err(e) => format!("[Bridge Error] {}", e),
        }
    }

    pub fn get_status(&self) -> String {
        "Unia Core: Wasm Runtime | UPA Virtualization: ACTIVE".to_string()
    }
}
