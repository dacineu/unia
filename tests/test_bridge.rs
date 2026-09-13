#[cfg(test)]
mod tests {
    use askillify::bridge::IntelligenceBridge;
    use askillify::registry::ActuatorRegistry;
    use askillify::router::RouterSlm;
    use askillify::transducer::{SynapticTransducer, TransductionLayer};
    use std::fs;

    #[test]
    fn test_intelligence_bridge_evolution_loop() {
        let registry = ActuatorRegistry::new("simulated_db");
        let router = RouterSlm::new("phi-3-router");
        let transducer = SynapticTransducer::new(TransductionLayer::LocalSovereign);
        
        let mut bridge = IntelligenceBridge::new(
            registry, 
            router, 
            transducer, 
            "NVIDIA-NIM"
        );

        let prompt = "Rust Object Oriented Modeling".to_string();

        // 1. First request: External Fallback
        let res1 = bridge.request(prompt.clone());
        assert!(res1.contains("External response"));

        // 2. Repeat request to hit frequency threshold (3)
        bridge.request(prompt.clone());
        bridge.request(prompt.clone());

        // 3. Fourth request: Should be an Internal Hit
        let res4 = bridge.request(prompt.clone());
        assert!(res4.contains("[INTERNAL_ACTUATOR]"));
        println!("Bridge successfully short-circuited the request using evolved .ure!");
    }
}
