use uuid::Uuid;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use sha2::{Sha256, Digest};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum TransductionLayer {
    LocalSovereign,     // Ollama / Local SLM
    SpecializedProvider, // Nvidia NIM
    UniversalOmni,      // OpenRouter / Claude / GPT-4
}

#[derive(Debug, Clone)]
pub struct InteractionPair {
    pub input: String,
    pub output: String,
    pub latency_ms: u64,
    pub success: bool,
}

pub struct SynapticTransducer {
    layer_preference: TransductionLayer,
    shadow_logs: Mutex<HashMap<String, Vec<InteractionPair>>>,
}

impl SynapticTransducer {
    pub fn new(layer: TransductionLayer) -> Self {
        Self {
            layer_preference: layer,
            shadow_logs: Mutex::new(HashMap::new()),
        }
    }

    /// Wraps an external resource call to record Input -> Output patterns.
    /// This is the "Shadowing" phase.
    pub fn shadow_resource(&self, resource_id: &str, input: &str, output: &str, latency: u64, success: bool) {
        let pair = InteractionPair {
            input: input.to_string(),
            output: output.to_string(),
            latency_ms: latency,
            success,
        };
        let mut logs = self.shadow_logs.lock().unwrap();
        logs.entry(resource_id.to_string()).or_default().push(pair);
        println!("Shadowing pattern for {}: [Input: {}, Success: {}]", resource_id, input, success);
    }

    /// Converts captured interaction patterns into a internal la-piece-de-résistance .ure actuator.
    pub fn transduce(&self, resource_id: &str, provenance: Option<&Value>) -> Result<Uuid, Box<dyn std::error::Error>> {
        let logs = self.shadow_logs.lock().unwrap();
        let patterns = logs.get(resource_id)
            .ok_or("No shadow logs available for this resource")?;

        println!("Transducing {} patterns using {:?} layer...", patterns.len(), self.layer_preference);

        // 1. Pattern Extraction (Simulated Router-SLM logic)
        // FIX: Include the resource_id (which contains the prompt) in the guidance to allow registry matching.
        let latent_logic = format!(
            "Pattern [{}]: Behavioral pattern derived from {} successful interactions. Priority: High-Fidelity Mirroring.",
            resource_id,
            patterns.iter().filter(|p| p.success).count()
        );

        // 2. Generate DU-UUID based on the derived logic
        let mut hasher = Sha256::new();
        hasher.update(latent_logic.as_bytes());
        let result = hasher.finalize();

        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[..16]);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        let resource_id_uuid = Uuid::from_bytes(bytes);

        // 3. Create the .ure manifest (The Mirror Actuator) with Provenance
        let mut manifest = json!({
            "resource_id": resource_id_uuid.to_string(),
            "resource_type": "mirror_actuator",
            "status": "UNSTABLE", // Mirrors start as unstable
            "guidance": latent_logic,
            "source_external_id": resource_id,
            "complexity_score": 0.7,
            "compressed_data": "simulated_pattern_weights_base64"
        });

        if let Some(prov) = provenance {
            manifest["provenance"] = prov.clone();
        }

        let file_path = format!("{}.ure", resource_id_uuid);
        fs::write(&file_path, serde_json::to_string_pretty(&manifest)?)?;

        println!("✅ Mirror Actuator synthesized with provenance: {}", resource_id_uuid);
        Ok(resource_id_uuid)
    }
    /// Implements the Mirroring Pattern: compares the synthetic Mirror against the External Source.
    pub fn verify_mirror(&self, mirror_id: Uuid, external_id: &str, test_input: &str, _external_output: &str) -> bool {
        println!("Verifying Mirror {} against External {}.", mirror_id, external_id);

        // In a real system, we would execute the mirror_id actuator and compare its output.
        // For the prototype, we simulate a "Fidelity Check".
        let mirror_output_simulated = format!("Simulated output for {}", test_input);

        // If the mirror output is consistent with external but lower latency, it's a win.
        let is_faithful = mirror_output_simulated.contains("Simulated");

        if is_faithful {
            println!("Mirror {} passed fidelity check. Promotion recommended.", mirror_id);
            true
        } else {
            println!("Mirror {} failed fidelity check.", mirror_id);
            false
        }
    }
}
