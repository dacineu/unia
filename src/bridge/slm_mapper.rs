use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct SlmRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    pub options: SlmOptions,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlmOptions {
    pub temperature: f32,
    pub num_predict: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlmResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
}

/// The SemanticSLM is a client for a local Small Language Model (e.g. Ollama).
/// To remain browser-compatible, it uses a trait for the network transport.
pub struct SemanticSLM {
    pub endpoint: String,
    pub model_name: String,
}

impl SemanticSLM {
    pub fn new(endpoint: String, model_name: String) -> Self {
        Self { endpoint, model_name }
    }

    /// Generates the la-piece-de-résistance prompt for the SLM.
    pub fn build_prompt(&self, intent: &str, available_actions: &[String]) -> String {
        let actions_list = available_actions.join(", ");
        format!(
            "You are a unia Semantic Mapper. Map the following user intent to EXACTLY ONE of these action IDs: [{}].\n\nIntent: '{}'\n\nReturn ONLY the action ID, nothing else.",
            actions_list, intent
        )
    }

    /// For native execution, this would use reqwest. For browser, it uses web_sys::fetch.
    /// To keep the prototype clean, we provide the logic to be called by the runtime.
    pub async fn map_intent_semantic<F>(&self, intent: &str, available_actions: &[String], fetch_fn: F) -> Result<String, String> 
    where 
        F: Fn(String, String) -> Result<String, String> 
    {
        let prompt = self.build_prompt(intent, available_actions);
        
        // In a real system, we'd send the SlmRequest JSON. 
        // Here we call the provided fetch_fn to remain environment-agnostic.
        let response = fetch_fn(self.endpoint.clone(), prompt)?;
        
        // Extract the action ID from the response (simulated JSON parsing)
        Ok(response.trim().to_string())
    }
}
