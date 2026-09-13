#[cfg(test)]
mod tests {
    use askillify::orchestrator::MetaOrchestrator;
    use askillify::meta_actuators::MetaActuatorType;

    /*
    #[test]
    fn test_meta_orchestrator_fleet_fanout() {
        let mut meta = MetaOrchestrator::new();

        // Spawn specialized agents
        let id1 = meta.spawn_agent("Architect", "System Design", MetaActuatorType::Synthesizer);
        let id2 = meta.spawn_agent("Coder", "Rust Implementation", MetaActuatorType::Mutator);
        let id3 = meta.spawn_agent("Auditor", "Security Analysis", MetaActuatorType::Analyst);

        let prompt = "Implement a secure DU-UUID generator".to_string();
        let results = meta.fan_out(&prompt);

        assert_eq!(results.len(), 3);
        assert!(results.get(&id1).unwrap().contains("Architect"));
        assert!(results.get(&id2).unwrap().contains("Coder"));
        assert!(results.get(&id3).unwrap().contains("Auditor"));
    }
    */
}
