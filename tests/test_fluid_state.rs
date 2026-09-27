#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;
    use unia::fluid::{FluidStateProjector, MicroNucleus, SlotType};
    use unia::registry::ActuatorRegistry;
    use unia::router::RouterSlm;
    use uuid::Uuid;

    fn create_mock_ure(dir: &Path, id: Uuid, guidance: &str) {
        let manifest = json!({
            "resource_id": id.to_string(),
            "guidance": guidance,
            "complexity_score": 0.5
        });
        fs::write(
            dir.join(format!("{}.ure", id)),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn test_fluid_state_flux_swap() {
        // Manifests are written into a temporary directory and the registry is
        // pointed at it explicitly. Writing to the process working directory
        // littered the repository root with .ure files on every test run.
        let dir = tempfile::tempdir().unwrap();
        let registry = Arc::new(ActuatorRegistry::with_base_dir("simulated_db", dir.path()));
        let router = Arc::new(RouterSlm::new("phi-3-router"));

        let logic_id = Uuid::new_v4();
        let presenter_a = Uuid::new_v4();
        let presenter_b = Uuid::new_v4();

        create_mock_ure(dir.path(), logic_id, "Core Logic: Calculate Pi");
        create_mock_ure(dir.path(), presenter_a, "Interface: Detailed Table");
        create_mock_ure(dir.path(), presenter_b, "Interface: Minimalist Bar");

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
        projector
            .flux_swap(SlotType::Presenter, presenter_b)
            .expect("Swap failed");

        // 3. Second Execution
        let out2 = projector.execute("Run");
        assert!(out2.contains("Minimalist Bar"));
        assert!(out2.contains("Calculate Pi")); // Logic remains identical
    }
}
