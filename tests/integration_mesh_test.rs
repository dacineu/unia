#[cfg(test)]
mod integration_tests {
    use askillify::registry::ActuatorRegistry;
    use askillify::orchestrator::{Orchestrator, BehavioralVector};
    use askillify::slm::MockSlm;
    use serde_json::json;
    use uuid::Uuid;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_end_to_end_actuation_flow() {
        let dir = tempdir().unwrap();
        
        // 1. Setup: Create a .ure resource
        let res_id = Uuid::new_v4();
        let file_path = dir.path().join(format!("{}.ure", res_id));
        let manifest = json!({
            "resource_id": res_id.to_string(),
            "resource_type": "skill",
            "complexity_score": 0.4,
            "guidance": "Expert in income generation."
        });
        fs::write(&file_path, serde_json::to_string(&manifest).unwrap()).unwrap();

        // 2. Initialize components
        let registry = ActuatorRegistry::new("mock_db");
        let orchestrator = Orchestrator::new(registry);
        let slm = MockSlm::new();

        // 3. Scenario: User wants "Quickest" execution
        let activation = orchestrator.collapse_actuator(res_id, BehavioralVector::Quickest).unwrap();
        let response = slm.execute("Make me money", &activation).unwrap();

        assert!(response.text.contains("FAST_EXECUTION"));
        assert!(response.tokens_used <= activation.token_budget);
        println!("Quickest Path: {} tokens, steps: {}", response.tokens_used, response.reasoning_steps);

        // 4. Scenario: User wants "Smartest" execution
        let activation_smart = orchestrator.collapse_actuator(res_id, BehavioralVector::Smartest).unwrap();
        let response_smart = slm.execute("Make me money", &activation_smart).unwrap();

        assert!(response_smart.text.contains("DEEP_REASONING"));
        assert!(response_smart.reasoning_steps > 1);
        println!("Smartest Path: {} tokens, steps: {}", response_smart.tokens_used, response_smart.reasoning_steps);
    }
}
