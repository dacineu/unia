use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use askillify::bridge::primitive::{PrimitiveBridge, UreResource, UreAction, StateType};
use askillify::nucleus::{ActuatorNucleus, ValveDriver, FileSystemDriver, TempSensorDriver};
use askillify::wmis::{WmisEconomicLayer, WmisResource, ResourceType, SharingScope};
use askillify::orchestrator::MetaOrchestrator;
use askillify::wmis::WmisDiscoveryProvider;

fn main() {
    println!("\n🌟 Welcome to the unia Universal Demo: Autonomous Infrastructure Flow\n");

    // --- 1. INFRASTRUCTURE SETUP ---
    let economy = Arc::new(Mutex::new(WmisEconomicLayer::new()));
    let discovery = Arc::new(WmisDiscoveryProvider::new());

    // --- 2. RESOURCE DEFINITIONS (.ure manifests) ---

    // A. The Smart Valve
    let mut valve_state = HashMap::new();
    valve_state.insert("flow_rate".to_string(), StateType { r#type: "float".to_string(), range: Some((0.0, 1.0)), unit: Some("percentage".to_string()), values: None });
    let valve_ure = UreResource {
        ure_version: "1.0".to_string(),
        resource_id: "valve-001".to_string(),
        category: "actuator".to_string(),
        state_space: valve_state,
        action_primitives: vec![UreAction {
            id: "emergency_shutdown".to_string(),
            aliases: Some(vec!["stop the valve".to_string(), "close water".to_string()]),
            params: HashMap::new(),
            target_state: "flow_rate = 0.0".to_string(),
            constraints: vec![]
        }],
    };

    // B. The FileSystem
    let mut fs_state = HashMap::new();
    fs_state.insert("last_write".to_string(), StateType { r#type: "string".to_string(), range: None, unit: Some("path".to_string()), values: None });
    let fs_ure = UreResource {
        ure_version: "1.0".to_string(),
        resource_id: "fs-root-001".to_string(),
        category: "system".to_string(),
        state_space: fs_state,
        action_primitives: vec![UreAction {
            id: "write_file".to_string(),
            aliases: Some(vec!["dump logs".to_string(), "save file".to_string()]),
            params: HashMap::new(),
            target_state: "last_write = current_path".to_string(),
            constraints: vec![]
        }],
    };

    // C. The Temp Sensor
    let mut temp_state = HashMap::new();
    temp_state.insert("temp".to_string(), StateType { r#type: "float".to_string(), range: Some((-50.0, 100.0)), unit: Some("C".to_string()), values: None });
    let temp_ure = UreResource {
        ure_version: "1.0".to_string(),
        resource_id: "temp-001".to_string(),
        category: "sensor".to_string(),
        state_space: temp_state,
        action_primitives: vec![UreAction {
            id: "get_temp".to_string(),
            aliases: Some(vec!["check temperature".to_string(), "how hot is it".to_string()]),
            params: HashMap::new(),
            target_state: "output = current_temp".to_string(),
            constraints: vec![]
        }],
    };

    // Register in Bridge and Nucleus
    let mut bridge_setup = PrimitiveBridge::new();
    bridge_setup.load_resource(valve_ure);
    bridge_setup.load_resource(fs_ure);
    bridge_setup.load_resource(temp_ure);
    let bridge = Arc::new(bridge_setup);

    let mut nucleus_setup = ActuatorNucleus::new(Arc::clone(&economy));
    nucleus_setup.register_driver(Box::new(ValveDriver { id: "valve-001".to_string() }));
    nucleus_setup.register_driver(Box::new(FileSystemDriver { id: "fs-root-001".to_string() }));
    nucleus_setup.register_driver(Box::new(TempSensorDriver { id: "temp-001".to_string() }));
    let nucleus = Arc::new(nucleus_setup);

    let mut orchestrator = MetaOrchestrator::new(discovery, Arc::clone(&bridge), Arc::clone(&nucleus));

    // --- 3. THE UNIVERSAL FLOW ---
    let user = "operator_01";

    // We spawn one agent to handle everything
    let agent_id = orchestrator.spawn_agent("Infrastructure Manager", "Full-Stack Actuation", askillify::meta_actuators::MetaActuatorType::Synthesizer);

    let tasks = vec![
        ("valve-001", "Emergency shutdown the main water valve"),
        ("fs-root-001", "Please dump logs to the system"),
        ("temp-001", "How hot is it in the server room?"),
    ];

    println!("--------------------------------------------------");
    println!("🚀 Executing Heterogeneous Task Sequence...");
    println!("--------------------------------------------------");

    for (res_id, intent) in tasks {
        println!("\n🎯 Intent: '{}'", intent);

        let meta = WmisResource {
            id: res_id.to_string(),
            resource_type: ResourceType::Application,
            owner: user.to_string(),
            sharing_scope: SharingScope::Global,
            capabilities: vec![],
            quality: askillify::wmis::QualityMetrics { qor: 1.0, qos: 1.0, qop: 1.0 },
            metadata: serde_json::json!({}),
        };

        let packet = bridge.map_intent(res_id, intent).expect("Bridge mapping failed");

        let mut packet_mut = packet;
        if res_id == "fs-root-001" {
            packet_mut.payload.arguments.insert("path".to_string(), "/var/log/system.log".to_string());
            packet_mut.payload.arguments.insert("content".to_string(), "Critical: Overheat detected".to_string());
        }

        let result = nucleus.dispatch(packet_mut, user, &meta).expect("Nucleus dispatch failed");
        println!("✅ Result: {}", result);
    }

    println!("\n--------------------------------------------------");
    println!("🏆 Demo Complete: All disparate resources controlled via one pipeline!");
    println!("--------------------------------------------------");
}
