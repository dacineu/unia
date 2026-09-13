#[cfg(test)]
mod tests {
    use askillify::fluid::{FluidStateProjector, MicroNucleus, SlotType};
    use askillify::registry::ActuatorRegistry;
    use askillify::router::RouterSlm;
    use uuid::Uuid;
    use std::collections::HashMap;
    use std::sync::Arc;
    use serde_json::json;
    use std::fs;

    fn create_mock_ure(id: Uuid, guidance: &str) {
        let manifest = json!({
            "resource_id": id.to_string(),
            "guidance": guidance,
            "complexity_score": 0.5
        });
        fs::write(format!("{}.ure", id), serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    }

    #[test]
    fn test_fluid_state_flux_swap() {
        let registry = Arc::new(ActuatorRegistry::new("simulated_db"));
        let router = Arc::new(RouterSlm::new("phi-3-router"));
        
        let logic_id = Uuid::new_v4();
        let presenter_a = Uuid::new_v4();
        let presenter_b = Uuid::new_v4();

        create_mock_ure(logic_id, "Core Logic: Calculate Pi");
        create_mock_ure(presenter_a, "Interface: Detailed Table");
        create_mock_ure(presenter_b, "Interface: Minimalist Bar");

        let mut slots = HashMap::new();
        slots.insert(SlotType::Logic, logic_id);
        slots.insert(SlotType::Presenter, presenter_a);

        let nucleus = MicroNucleus {
            id: Uuid::new_v4(),
            slots,
            context_state: json!({}),
        };

        let projector = FluidStateProjector::new(registry, router, nucleus);

        // 1. Initial Execution
        let out1 = projector.execute("Run");
        assert!(out1.contains("Detailed Table"));

        // 2. The Flux Point: Swap Presenter only
        projector.flux_swap(SlotType::Presenter, presenter_b).expect("Swap failed");

        // 3. Second Execution
        let out2 = projector.execute("Run");
        assert!(out2.contains("Minimalist Bar"));
        assert!(out2.contains("Calculate Pi")); // Logic remains identical
    }
}
