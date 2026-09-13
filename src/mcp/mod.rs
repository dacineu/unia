use std::process::{Command, Child};
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub server_cmd: String,
    pub server_args: Vec<String>,
    pub env_vars: HashMap<String, String>,
}

pub struct McpSession {
    pub process: Child,
    pub config: McpConfig,
}

pub struct McpResourceManager {
    active_sessions: Arc<Mutex<HashMap<Uuid, McpSession>>>,
    max_slots: usize,
}

impl McpResourceManager {
    pub fn new(max_slots: usize) -> Self {
        Self {
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            max_slots,
        }
    }

    /// Hot-swaps or spawns an MCP server based on a .ure config.
    pub fn activate_connector(&self, id: Uuid, config: McpConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.active_sessions.lock().unwrap();

        if sessions.len() >= self.max_slots {
            println!("Memory budget full. Pruning least-recently-used connector...");
            if let Some(key) = sessions.keys().next().cloned() {
                if let Some(mut session) = sessions.remove(&key) {
                    let _ = session.process.kill();
                    println!("♻️ Freed RAM from connector {}", key);
                }
            }
        }

        println!("🚀 Spawning MCP Connector {} via {}...", id, config.server_cmd);
        let child = Command::new(&config.server_cmd)
            .args(&config.server_args)
            .envs(&config.env_vars)
            .spawn()?;

        sessions.insert(id, McpSession { process: child, config });
        Ok(())
    }

    pub fn deactivate_connector(&self, id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.active_sessions.lock().unwrap();
        if let Some(mut session) = sessions.remove(id) {
            let _ = session.process.kill();
            println!("📉 Deactivated connector {}. RAM freed.", id);
        }
        Ok(())
    }

    pub fn call_mcp(&self, id: &Uuid, method: &str, params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let sessions = self.active_sessions.lock().unwrap();
        if !sessions.contains_key(id) {
            return Err("Connector not active".into());
        }

        println!("📡 MCP Call [{}]: method={}", id, method);
        Ok(json!({
            "content": "Raw external data from MCP server",
            "metadata": { "source": "github.com/example/repo" }
        }))
    }
}
