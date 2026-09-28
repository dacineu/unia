#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use unia::harvester::ExplorerHarvester;
    use unia::learner::{Learner, TrainingBackend};
    use unia::pipeline::EvolutionaryPipeline;
    use unia::registry::ActuatorRegistry;
    use unia::router::RouterSlm;
    use unia::transducer::{SynapticTransducer, TransductionLayer};

    #[tokio::test]
    async fn test_evolutionary_pipeline_flow() {
        let registry = Arc::new(ActuatorRegistry::new("simulated_db"));
        let _router = Arc::new(RouterSlm::new("phi-3-router"));
        let transducer = Arc::new(SynapticTransducer::new(TransductionLayer::LocalSovereign));
        let learner = Arc::new(Learner::new(TrainingBackend::Local("ollama".to_string())));
        let harvester = Arc::new(ExplorerHarvester::new(
            ActuatorRegistry::new("simulated_db"),
            RouterSlm::new("phi-3-router"),
        ));

        let pipeline = EvolutionaryPipeline::new(harvester, learner, transducer, registry);

        let source = "https://github.com/stablyai/orca";
        let result = pipeline.evolve_external_resource(source).await;

        assert!(result.is_ok());
        let ure_id = result.unwrap();
        println!("Pipeline evolved resource into .ure: {}", ure_id);
    }
}
