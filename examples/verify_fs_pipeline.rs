use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use askillify::bridge::primitive::{PrimitiveBridge, UreResource, UreAction, StateType};
use askillify::nucleus::{ActuatorNucleus, FileSystemDriver};
use askillify::wmis::{WmisEconomicLayer, WmisResource, ResourceType, SharingScope};

fn main() {
    println!("\n🧪 Verifying unia Pipeline: Intent -> Bridge -> Nucleus (FileSystem)\n");

    let mut bridge = PrimitiveBridge::new();
    let economy = Arc::new(Mutex::new(WmisEconomicLayer::new()));
    let mut nucleus = ActuatorNucleus::new(Arc::clone(&economy));

    // 1. Load the FS Resource
    let mut state_space = HashMap::new();
    state_space.insert("last_write".to_string(), StateType {
        r#type: "string".to_string(),
        range: None,
        unit: Some("path".to_string()),
        values: None,
    });

    let actions = vec![
        UreAction {
            id: "write_file".to_string(),
            aliases: Some(vec!["save file".to_string(), "dump logs".to_string()]),
            params: HashMap::new(),
            target_state: "last_write = current_path".to_string(),
            constraints: vec!["disk_space > 0".to_string()],
        },
    ];

    let fs_ure = UreResource {
        ure_version: "1.0".to_string(),
        resource_id: "fs-root-001".to_string(),
        category: "system".to_string(),
        state_space,
        action_primitives: actions,
    };

    bridge.load_resource(fs_ure);
    nucleus.register_driver(Box::new(FileSystemDriver { id: "fs-root-001".to_string() }));

    let fs_meta = WmisResource {
        id: "fs-root-001".to_string(),
        resource_type: ResourceType::Storage,
        owner: "test_user".to_string(),
        sharing_scope: SharingScope::Global,
        capabilities: vec!["fs_access".to_string()],
        quality: askillify::wmis::QualityMetrics { qor: 1.0, qos: 1.0, qop: 1.0 },
        metadata: serde_json::json!({}),
    };

    // --- TEST: Using an Alias ---
    let intent = "Please dump logs to the system";
    println!("Intent: '{}'", intent);

    let packet = bridge.map_intent("fs-root-001", intent).expect("Bridge mapping failed");
    
    // Simulate adding arguments that would normally come from an LLM/Agent
    let mut packet_mut = packet;
    packet_mut.payload.arguments.insert("path".to_string(), "/var/log/system.log".to_string());
    packet_mut.payload.arguments.insert("content".to_string(), "ERROR: Disk full".to_string());

    let result = nucleus.dispatch(packet_mut, "test_user", &fs_meta).expect("Nucleus dispatch failed");
    println!("Result: {}", result);
    println!("\n✅ Pipeline verified for FileSystem resource using semantic aliases!");
}
