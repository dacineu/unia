use crate::registry::ActuatorRegistry;
use crate::router::RouterSlm;
use crate::orchestrator::BehavioralVector;
use crate::transducer::SynapticTransducer;
use std::collections::HashMap;

/// Simulated External API (e.g., NVIDIA NIM)
pub struct ExternalApi {
    pub provider: String,
}

impl ExternalApi {
    pub fn call(&self, prompt: &str) -> String {
        println!("[External API - {}] Processing: {}", self.provider, prompt);
        format!("External response to '{}' using la-piece-de-résistance logic.", prompt)
    }
}

pub struct IntelligenceBridge {
    registry: ActuatorRegistry,
    router: RouterSlm,
    transducer: SynapticTransducer,
    external_api: ExternalApi,
    pattern_frequency: HashMap<String, usize>,
}

impl IntelligenceBridge {
    pub fn new(registry: ActuatorRegistry, router: RouterSlm, transducer: SynapticTransducer, provider: &str) -> Self {
        Self {
            registry,
            router,
            transducer,
            external_api: ExternalApi { provider: provider.to_string() },
            pattern_frequency: HashMap::new(),
        }
    }

    /// The la-piece-de-résistance interceptor.
    /// Orchestrates: Intercept -> Intervene -> Evolve.
    pub fn request(&mut self, prompt: String) -> String {
        // 1. INTERCEPT: Check for internal prototype
        let matches = self.registry.find_matching_actuators(&prompt);
        
        if !matches.is_empty() {
            println!("🎯 Internal Hit! Intercepting request with pre-compiled Actuators.");
            
            // PREFER CHAMPION: If a champion exists for this prompt, use it exclusively
            let champion_id = self.registry.get_champion(&prompt);
            let selected_actuators = if let Some(cid) = champion_id {
                println!("🏆 Using Champion Actuator: {}", cid);
                matches.into_iter().filter(|(id, _)| *id == cid).collect()
            } else {
                matches
            };

            // 2. INTERVENE: Synthesize an internal response
            if !selected_actuators.is_empty() {
                if let Ok(hybrid) = self.router.synthesize(selected_actuators, BehavioralVector::Smartest) {
                    let activation = hybrid.to_activation_vector(BehavioralVector::Smartest);
                    return format!("[INTERNAL_ACTUATOR] Result based on {}: {}", 
                        hybrid.source_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "),
                        activation.system_prompt);
                }
            }
        }

        // 3. EXTERNAL FALLBACK: Forward to LLM
        println!("🌐 No internal prototype found. Routing to external API...");
        let response = self.external_api.call(&prompt);

        // 4. EVOLVE: Shadow and potentially transduce
        self.evolve_pattern(&prompt, &response);

        response
    }

    fn evolve_pattern(&mut self, prompt: &str, response: &str) {
        let pattern_id = format!("{}:{}", prompt, response); // Simplified pattern ID
        
        // Shadow the interaction
        self.transducer.shadow_resource(
            &pattern_id, 
            prompt, 
            response, 
            150, // simulated latency
            true
        );

        // Track frequency for crystallization
        let count = self.pattern_frequency.entry(pattern_id.clone()).or_insert(0);
        *count += 1;

        if *count >= 3 {
            println!("✨ Pattern frequency reached threshold. Crystallizing into .ure Actuator...");
            if let Err(e) = self.transducer.transduce(&pattern_id, None) {
                eprintln!("Transduction error: {}", e);
            }
        }
    }
}
pub mod primitive;
#[cfg(test)]
mod primitive_tests;
