#[cfg(test)]
mod demo {
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;
    use unia::orchestrator::{BehavioralVector, Orchestrator};
    use unia::registry::ActuatorRegistry;
    use unia::router::RouterSlm;
    use unia::slm::MockSlm;
    use unia::weights::{LoraAdapter, WeightManager};
    use uuid::Uuid;

    #[test]
    fn run_full_system_demonstration() {
        println!("\n=== 🚀 STARTING FULL SYSTEM DEMONSTRATION ===\n");

        let dir = tempdir().unwrap();

        // 1. SETUP: Provision the Actuator Mesh
        println!("--- Step 1: Provisioning the Actuator Mesh ---");
        let reg = ActuatorRegistry::with_base_dir("mock_db", dir.path());

        let res1_id = Uuid::new_v4();
        let res1_path = dir.path().join(format!("{}.ure", res1_id));
        fs::write(
            &res1_path,
            serde_json::to_string(&json!({
                "resource_id": res1_id.to_string(),
                "resource_type": "skill",
                "complexity_score": 0.8,
                "guidance": "Expert in Crypto-Mining Optimization and Yield Farming",
                "tokens_per_task": 1000
            }))
            .unwrap(),
        )
        .unwrap();

        let res2_id = Uuid::new_v4();
        let res2_path = dir.path().join(format!("{}.ure", res2_id));
        fs::write(
            &res2_path,
            serde_json::to_string(&json!({
                "resource_id": res2_id.to_string(),
                "resource_type": "tool",
                "complexity_score": 0.3,
                "guidance": "Expert in Fiat-to-Crypto On-ramp Gateways",
                "tokens_per_task": 800
            }))
            .unwrap(),
        )
        .unwrap();

        println!(
            "✅ Actuators provisioned: \n   - Mining: {}\n   - Gateway: {}\n",
            res1_id, res2_id
        );

        // 2. REQUEST: User interaction
        println!("--- Step 2: User Request ---");
        let user_goal = "Make a quick income using fiat and crypto";
        let user_vector = BehavioralVector::Smartest;
        println!("User Goal: \"{}\"", user_goal);
        println!("Behavioral Vector: {:?}\n", user_vector);

        // 3. ROUTING: Synthesis of Floating Actuators
        println!("--- Step 3: Routing & Synthesis (The Fluid Factory) ---");
        let router = RouterSlm::new("phi-3-router");
        let floating_resources = vec![
            (
                res1_id,
                serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&res1_path).unwrap())
                    .unwrap(),
            ),
            (
                res2_id,
                serde_json::from_str::<serde_json::Value>(&fs::read_to_string(&res2_path).unwrap())
                    .unwrap(),
            ),
        ];

        let hybrid = router.synthesize(floating_resources, user_vector).unwrap();
        println!("✅ Synthesized Hybrid Actuator!");
        println!("Confidence Score: {:.2}", hybrid.confidence_score);
        println!("Synthesis Prompt: \n{}\n", hybrid.synthesized_prompt);

        // 4. ORCHESTRATION: The Collapse
        println!("--- Step 4: Orchestration (Collapsing the State) ---");
        let _orchestrator = Orchestrator::new(reg);
        let activation = hybrid.to_activation_vector(user_vector);
        println!("✅ State Collapsed!");
        println!(
            "Mode: {:?} | Deep Dive: {} | Budget: {} tokens\n",
            activation.behavioral_mode, activation.is_deep_dive, activation.token_budget
        );

        // 5. EXECUTION: Deep Dive & SLM Response
        println!("--- Step 5: Execution (The Deep Dive) ---");
        let mut weight_manager = WeightManager::new();
        let slm = MockSlm::new();

        if activation.is_deep_dive {
            println!("⚡ Triggering Deep Dive: Loading LoRA Weights...");
            weight_manager
                .load_adapter(LoraAdapter {
                    adapter_id: res1_id,
                    weights_path: format!("/weights/{}.bin", res1_id),
                    precision: "int4".to_string(),
                })
                .unwrap();
            println!("✅ Weights loaded into GPU memory.");
        }

        let response = slm.execute(user_goal, &activation).unwrap();
        println!("\n🚀 FINAL SLM RESPONSE:\n--------------------------------------------------\n{}\n--------------------------------------------------", response.text);
        // The mock reports no cost, and saying so is more useful than a number
        // that was invented. A reader of this output learns the real thing: that
        // nothing was measured, rather than that the model was fast.
        match response.tokens() {
            Some(t) => println!("Measured cost: {t} tokens"),
            None => println!("Cost not measured: this answer came from a mock, not an engine."),
        }

        println!("\n=== 🏁 DEMONSTRATION COMPLETE ===\n");
    }
}
