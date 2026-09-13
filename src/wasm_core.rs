use wasm_bindgen::prelude::*;
use std::sync::{Arc, Mutex};
use crate::bridge::primitive::{PrimitiveBridge, UreResource, UreAction, StateType};
use crate::nucleus::ActuatorNucleus;
use crate::wmis::{WmisDiscoveryProvider, WmisResource, ResourceType, SharingScope, QualityMetrics};
use crate::os::{UniaKernel, UniaSyscall};
use std::collections::HashMap;

#[wasm_bindgen]
pub struct UniaCore {
    orchestrator: crate::orchestrator::MetaOrchestrator,
    kernel: UniaKernel,
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

        let bridge = Arc::new(bridge_setup);
        let orchestrator = crate::orchestrator::MetaOrchestrator::new(
            discovery,
            Arc::clone(&bridge),
            Arc::clone(&nucleus)
        );

        let kernel = UniaKernel::new(bridge, nucleus);

        Self { orchestrator, kernel }
    }

    pub fn execute_os_command(&mut self, user: &str, command: &str) -> String {
        // Simple OS Shell logic: "write /dev/unia/upa-virtual-01 100"
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return "No command provided".to_string(); }

        match parts[0] {
            "write" if parts.len() >= 3 => {
                let path = parts[1];
                let val = parts[2];
                // Resolve path through VFS simulation
                let res_id = if path.contains("upa-virtual-01") { "upa-virtual-01" } else { "unknown" };
                let syscall = UniaSyscall::WriteDevice(res_id.to_string(), val.to_string());
                self.kernel.handle_syscall(syscall, user)
            },
            "compute" if parts.len() >= 2 => {
                let op = parts[1];
                let syscall = UniaSyscall::ExecuteCompute(op.to_string(), vec!["A".into(), "B".into()]);
                self.kernel.handle_syscall(syscall, user)
            },
            "status" => self.get_status(),
            _ => format!("Unknown OS command: {}. Try 'write /dev/unia/upa-virtual-01 100' or 'compute sum'", parts[0]),
        }
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
        "Unia OS: Wasm Kernel ACTIVE | UPA Fabric: ONLINE".to_string()
    }
}
