use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use crate::bridge::primitive::{PrimitivePacket, UniversalPrimitive};
use crate::nucleus::{ActuatorNucleus, ValveDriver};
use crate::wmis::{WmisEconomicLayer, WmisResource, ResourceType, SharingScope};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// A simple unia-node that listens for remote PrimitivePackets and executes them locally.
pub struct UniaNode {
    pub addr: String,
    pub nucleus: Arc<ActuatorNucleus>,
}

impl UniaNode {
    pub fn new(addr: String, nucleus: Arc<ActuatorNucleus>) -> Self {
        Self { addr, nucleus }
    }

    pub fn listen(&self) -> Result<(), String> {
        let listener = TcpListener::bind(&self.addr)
            .map_err(|e| format!("Failed to bind to {}: {}", self.addr, e))?;
        
        println!("📡 unia-node listening on {}", self.addr);

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let nucleus = Arc::clone(&self.nucleus);
                    std::thread::spawn(move || {
                        let mut buffer = [0; 1024];
                        if let Ok(n) = s.read(&mut buffer) {
                            let payload = String::from_utf8_lossy(&buffer[..n]);
                            if let Ok(packet) = serde_json::from_str::<PrimitivePacket>(&payload) {
                                // In a real node, we'd use a real user and resource meta
                                let mock_meta = WmisResource {
                                    id: packet.payload.resource_id.clone(),
                                    resource_type: ResourceType::Application,
                                    owner: "remote_user".to_string(),
                                    sharing_scope: SharingScope::Global,
                                    capabilities: vec![],
                                    quality: crate::wmis::QualityMetrics { qor: 1.0, qos: 1.0, qop: 1.0 },
                                    metadata: serde_json::json!({}),
                                };
                                
                                match nucleus.dispatch(packet, "remote_user", &mock_meta) {
                                    Ok(res) => { let _ = s.write_all(res.as_bytes()); },
                                    Err(e) => { let _ = s.write_all(e.as_bytes()); },
                                }
                            }
                        }
                    });
                },
                Err(e) => eprintln!("Connection error: {}", e),
            }
        }
        Ok(())
    }
}
