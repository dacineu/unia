#[cfg(test)]
mod tests {
    use askillify::registry::ActuatorRegistry;
    use askillify::orchestrator::{Orchestrator, BehavioralVector};
    use askillify::router::RouterSlm;
    use askillify::slm::MockSlm;
    use askillify::weights::{WeightManager, LoraAdapter};
    use askillify::learner::{Learner, TrainingBackend};
    use uuid::Uuid;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_the_complete_fluid_factory_loop() {
        println!("\n=== 🌟 STARTING LA-PIECE-DE-RÉSISTANCE FULL SYSTEM DEMO 🌟 ===\n");
        let dir = tempdir().unwrap();

        // 1. PROVISIONING: Create a few base actuators (The "Vocabulary")
        let reg = ActuatorRegistry::new("mock_db");
        let res_id = Uuid::new_v4();
        let path = dir.path().join(format!("{}.ure", res_id));
        fs::write(&path, serde_json::to_string(&json!({
            "resource_id": res_id.to_string(),
            "resource_type": "skill",
            "complexity_score": 0.8,
            "guidance": "Expert in Crypto-Yield Strategies",
            "tokens_per_task": 1000
        })).unwrap()).unwrap();

        println!("✅ Resource Provisioned: {} (Complexity: 0.8)\n", res_id);

        // 2. ORCHESTRATION: User asks for "Smartest" income path
        let orchestrator = Orchestrator::new(reg);
        let vector = BehavioralVector::Smartest;
        let activation = orchestrator.collapse_actuator(res_id, vector).unwrap();
        println!("✅ State Collapsed: Mode={:?}, DeepDive={}", activation.behavioral_mode, activation.is_deep_dive);

        // 3. EXECUTION: Deep Dive with Weights
        let mut weight_manager = WeightManager::new();
        let slm = MockSlm::new();

        if activation.is_deep_dive {
            weight_manager.load_adapter(LoraAdapter {
                adapter_id: res_id,
                weights_path: "/weights/core.bin".to_string(),
                precision: "int4".to_string(),
            }).unwrap();
            println!("⚡ LoRA Weights Loaded into GPU Memory.");
        }

        let response = slm.execute("Generate income", &activation).unwrap();
        println!("🚀 SLM Response: {}\n", response.text);

        // 4. LEARNING: Triggering the Learner because we want to "Optimize" this path
        let learner = Learner::new(TrainingBackend::Local("ollama".to_string()));
        let evolved_id = tokio::runtime::Runtime::new().unwrap().block_on(async {
            learner.train_specialization("User requested Smartest, result was successful but slow", vec![res_id]).await
        }).unwrap();

        println!("✅ Evolution Triggered! New Specialized Actuator evolved: {}", evolved_id);
        println!("\n=== 🏁 DEMONSTRATION COMPLETE: The la-piece-de-résistance la-piece-de-résistance is operational ===\n");
    }
}
