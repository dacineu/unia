use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LoraAdapter {
    pub adapter_id: Uuid,
    pub weights_path: String,
    pub precision: String, // "fp16", "int8", "int4"
}

pub struct WeightManager {
    // In production, this would interface with a runtime like LoRAX or vLLM
    // that manages GPU memory and adapter swapping.
    active_adapters: HashMap<Uuid, LoraAdapter>,
    gpu_memory_usage: usize,
}

impl WeightManager {
    pub fn new() -> Self {
        Self {
            active_adapters: HashMap::new(),
            gpu_memory_usage: 0,
        }
    }

    /// Simulates loading a LoRA adapter into GPU memory for a "Deep Dive".
    pub fn load_adapter(&mut self, adapter: LoraAdapter) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading LoRA adapter {} from {}...", adapter.adapter_id, adapter.weights_path);
        
        let adapter_size = match adapter.precision.as_str() {
            "int4" => 100 * 1024 * 1024, // 100MB
            "int8" => 200 * 1024 * 1024, // 200MB
            _ => 400 * 1024 * 1024,      // 400MB
        };

        if self.gpu_memory_usage + adapter_size > 8 * 1024 * 1024 * 1024 { // 8GB limit
            return Err("GPU_OUT_OF_MEMORY: Cannot load more adapters".into());
        }

        self.active_adapters.insert(adapter.adapter_id, adapter);
        self.gpu_memory_usage += adapter_size;
        
        Ok(())
    }

    /// Unloads an adapter to free memory.
    pub fn unload_adapter(&mut self, id: &Uuid) {
        if let Some(adapter) = self.active_adapters.remove(id) {
            let size = match adapter.precision.as_str() {
                "int4" => 100 * 1024 * 1024,
                "int8" => 200 * 1024 * 1024,
                _ => 400 * 1024 * 1024,
            };
            self.gpu_memory_usage -= size;
        }
    }

    pub fn is_loaded(&self, id: &Uuid) -> bool {
        self.active_adapters.contains_key(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_lifecycle() {
        let mut manager = WeightManager::new();
        let adapter = LoraAdapter {
            adapter_id: Uuid::new_v4(),
            weights_path: "/path/to/lora.bin".to_string(),
            precision: "int4".to_string(),
        };

        assert!(manager.load_adapter(adapter.clone()).is_ok());
        assert!(manager.is_loaded(&adapter.adapter_id));
        
        manager.unload_adapter(&adapter.adapter_id);
        assert!(!manager.is_loaded(&adapter.adapter_id));
    }
}
