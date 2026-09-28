#[cfg(test)]
mod integration_tests {
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;
    use unia::orchestrator::{BehavioralVector, Orchestrator};
    use unia::registry::ActuatorRegistry;
    use unia::slm::MockSlm;
    use uuid::Uuid;

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
        let registry = ActuatorRegistry::with_base_dir("mock_db", dir.path());
        let orchestrator = Orchestrator::new(registry);
        let slm = MockSlm::new();

        // 3. Scenario: User wants "Quickest" execution
        let activation = orchestrator
            .collapse_actuator(res_id, BehavioralVector::Quickest)
            .unwrap();
        let response = slm.execute("Make me money", &activation).unwrap();

        // The mode is visible in the answer, which is what this test is for.
        assert!(response.text.contains("[direct]"));
        // **No budget assertion, because there is no cost to check.** The old
        // test asserted `tokens_used <= token_budget` against a number the mock
        // had invented, which passed by arithmetic fiction: the comparison was
        // between two fabricated quantities. `None` is the honest answer and it
        // makes the check disappear rather than pass.
        assert_eq!(
            response.tokens(),
            None,
            "a mock cannot exceed a budget it never incurred"
        );
        println!("Quickest Path: cost not measured (mock)");

        // 4. Scenario: User wants "Smartest" execution
        let activation_smart = orchestrator
            .collapse_actuator(res_id, BehavioralVector::Smartest)
            .unwrap();
        let response_smart = slm.execute("Make me money", &activation_smart).unwrap();

        assert!(response_smart.text.contains("[analysis -> validation]"));
        // The old assertion here was `reasoning_steps > 1`. It passed when the
        // fabrication was convincing, so it was a test of the lie rather than of
        // the system, and no correct implementation could satisfy it. What
        // replaces it says what is actually true: the deeper mode is observable
        // in the answer, and neither mode claims a cost for the difference.
        assert_ne!(
            response_smart.text, response.text,
            "the two modes differ, which is the property the configuration affects"
        );
        assert_eq!(response_smart.tokens(), None, "and neither claims a price");
        println!("Smartest Path: cost not measured (mock)");
    }
}
