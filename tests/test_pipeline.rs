#[cfg(test)]
mod tests {
    use askillify::pipeline::EvolutionaryPipeline;
    use askillify::registry::ActuatorRegistry;
    use askillify::router::RouterSlm;
    use askillify::transducer::{SynapticTransducer, TransductionLayer};
    use askillify::learner::{Learner, TrainingBackend};
    use askillify::harvester::ExplorerHarvester;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_evolutionary_pipeline_flow() {
        let registry = Arc::new(ActuatorRegistry::new("simulated_db"));
        let router = Arc::new(RouterSlm::new("phi-3-router"));
        let transducer = Arc::new(SynapticTransducer::new(TransductionLayer::LocalSovereign));
        let learner = Arc::new(Learner::new(TrainingBackend::Local("ollama".to_string())));
        let harvester = Arc::new(ExplorerHarvester::new(ActuatorRegistry::new("simulated_db"), RouterSlm::new("phi-3-router")));

        let pipeline = EvolutionaryPipeline::new(
            harvester,
            learner,
            transducer,
            registry,
        );

        let source = "https://github.com/stablyai/orca";
        let result = pipeline.evolve_external_resource(source).await;

        assert!(result.is_ok());
        let ure_id = result.unwrap();
        println!("Pipeline evolved resource into .ure: {}", ure_id);
    }
}
