use askillify::registry::ActuatorRegistry;
use askillify::orchestrator::{Orchestrator, BehavioralVector};
use askillify::router::RouterSlm;
use askillify::slm::MockSlm;
use askillify::weights::{WeightManager, LoraAdapter};
use uuid::Uuid;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 🚀 Welcome to the Predictive Resource Orchestration Mesh Demo ===\n");
    
    let dir = tempdir()?;
    
    // 1. SETUP: Create the "Vocabulary" (Actuators in the Mesh)
    println!("--- Step 1: Provisioning the Actuator Mesh ---");
    let reg = ActuatorRegistry::new("mock_db");
    
    let res1_id = Uuid::new_v4();
    let res1_path = dir.path().join(format!("{}.ure", res1_id));
    fs::write(&res1_path, serde_json::to_string(&json!({
        "resource_id": res1_id.to_string(),
        "resource_type": "skill",
        "complexity_score": 0.8, // Deep Resource
        "guidance": "Expert in Crypto-Mining Optimization and Yield Farming",
        "tokens_per_task": 1000
    }))?)?;

    let res2_id = Uuid::new_v4();
    let res2_path = dir.path().join(format!("{}.ure", res2_id));
    fs::write(&res2_path, serde_json::to_string(&json!({
        "resource_id": res2_id.to_string(),
        "resource_type": "tool",
        "complexity_score": 0.3, // Surface Resource
        "guidance": "Expert in Fiat-to-Crypto On-ramp Gateways",
        "tokens_per_task": 800
    }))?)?;
    
    println!("✅ Actuators provisioned: \n   - Mining: {}\n   - Gateway: {}\n", res1_id, res2_id);

    // 2. REQUEST: User interaction
    println!("--- Step 2: User Request ---");
    let user_goal = "Make a quick income using fiat and crypto";
    let user_vector = BehavioralVector::Smartest; 
    println!("User Goal: '{}'", user_goal);
    println!("Behavioral Vector: {:?}\n", user_vector);

    // 3. ROUTING: Synthesis of Floating Actuators
    println!("--- Step 3: Routing & Synthesis (The Fluid Factory) ---");
    let router = RouterSlm::new("phi-3-router");
    let floating_resources = vec![
        (res1_id, serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&res1_path)?).unwrap()),
        (res2_id, serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&res2_path)?).unwrap()),
    ];
    
    let hybrid = router.synthesize(floating_resources, user_vector)?;
    println!("✅ Synthesized Hybrid Actuator!");
    println!("Confidence Score: {:.2}", hybrid.confidence_score);
    println!("Synthesis Prompt: \n{}\n", hybrid.synthesized_prompt);

    // 4. ORCHESTRATION: The Collapse
    println!("--- Step 4: Orchestration (Collapsing the State) ---");
    let orchestrator = Orchestrator::new(reg);
    let activation = hybrid.to_activation_vector(user_vector);
    println!("✅ State Collapsed!");
    println!("Mode: {:?} | Deep Dive: {} | Budget: {} tokens\n", 
        activation.behavioral_mode, activation.is_deep_dive, activation.token_budget);

    // 5. EXECUTION: Deep Dive & SLM Response
    println!("--- Step 5: Execution (The Deep Dive) ---");
    let mut weight_manager = WeightManager::new();
    let slm = MockSlm::new();

    if activation.is_deep_dive {
        println!("⚡ Triggering Deep Dive: Loading LoRA Weights...");
        weight_manager.load_adapter(LoraAdapter {
            adapter_id: res1_id,
            weights_path: format!("/weights/{}.bin", res1_id),
            precision: "int4".to_string(),
        })?;
        println!("✅ Weights loaded into GPU memory.");
    }

    let response = slm.execute(user_goal, &activation)?;
    println!("\n🚀 FINAL SLM RESPONSE:\n--------------------------------------------------\n{}\n--------------------------------------------------", response.text);
    println!("Metrics: {} tokens used, {} reasoning steps.", response.tokens_used, response.reasoning_steps);

    Ok(())
}
