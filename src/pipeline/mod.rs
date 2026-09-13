use crate::harvester::ExplorerHarvester;
use crate::learner::Learner;
use crate::transducer::SynapticTransducer;
use crate::registry::ActuatorRegistry;
use uuid::Uuid;
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum PipelineStage {
    Harvesting,
    Learning,
    Transducing,
}

pub struct EvolutionaryPipeline {
    harvester: Arc<ExplorerHarvester>,
    learner: Arc<Learner>,
    transducer: Arc<SynapticTransducer>,
    registry: Arc<ActuatorRegistry>,
}

impl EvolutionaryPipeline {
    pub fn new(
        harvester: Arc<ExplorerHarvester>,
        learner: Arc<Learner>,
        transducer: Arc<SynapticTransducer>,
        registry: Arc<ActuatorRegistry>,
    ) -> Self {
        Self {
            harvester,
            learner,
            transducer,
            registry,
        }
    }

    /// The la-piece-de-résistance unified learning flow.
    /// Raw Resource $\rightarrow$ Capability Seed $\rightarrow$ Specialized Actuator $\rightarrow$ .ure
    pub async fn evolve_external_resource(&self, source_path: &str) -> Result<Uuid, Box<dyn std::error::Error>> {
        println!("🌀 Initiating Evolutionary Pipeline for: {}", source_path);

        // 1. HARVESTING Stage
        println!("Stage 1: Harvesting Capability Seeds...");
        let seed_guidance = format!("Extracted seed from {}", source_path);
        
        // 2. LEARNING Stage
        println!("Stage 2: Agentic Learning & Specialization...");
        let task_trace = format!("Source: {}\nSeed: {}", source_path, seed_guidance);
        let base_actuators = Vec::new(); 
        let specialized_id = self.learner.train_specialization(&task_trace, base_actuators).await?;

        // 3. TRANSDUCING Stage
        println!("Stage 3: Transducing to la-piece-de-résistance Mattern...");
        
        let pattern_id = format!("evolve:{}", specialized_id);
        
        // Use the now-thread-safe shadow_resource
        self.transducer.shadow_resource(
            &pattern_id, 
            &seed_guidance, 
            &format!("Crystallized logic for {}", source_path), 
            100, 
            true
        );

        let final_ure_id = self.transducer.transduce(&pattern_id, None)?;
        
        println!("✅ Evolution Complete. Resource crystallized as: {}", final_ure_id);
        Ok(final_ure_id)
    }
}
