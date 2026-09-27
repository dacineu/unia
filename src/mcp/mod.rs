use uuid::Uuid;
use serde_json::{json, Value};
use std::process::{Child, Command};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub mod store;

/// Retrieval-quality harness. Measures whether routing is *correct*, which
/// latency and token counts cannot.
pub mod eval;

#[cfg(feature = "mcp-server")]
pub mod server;

#[cfg(feature = "mcp-server")]
pub use server::UniaServer;

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
            Ok(())
        } else {
            Err("Connector is not active".into())
        }
    }

    /// Forwards a JSON-RPC call to a spawned connector.
    ///
    /// This does not yet speak JSON-RPC: the wire framing and `initialize`
    /// handshake are unimplemented, so it cannot be used to reach a real server.
    /// Callers should treat it as unavailable rather than as a working shim.
    pub fn call_mcp(&self, id: &Uuid, method: &str, _params: Value) -> Result<Value, Box<dyn std::error::Error>> {
        let sessions = self.active_sessions.lock().unwrap();
        if !sessions.contains_key(id) {
            return Err("Connector is not active".into());
        }

        Err(format!(
            "MCP transport not implemented: cannot send {method} to connector {id}. \
             Use the unia-mcp server binary and connect a client to it over stdio instead."
        )
        .into())
    }
}
