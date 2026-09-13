use crate::bridge::primitive::{PrimitivePacket, UniversalPrimitive};
use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Write, Read};

/// The NetworkActuatorDriver allows a .ure resource to be hosted on a remote machine.
/// It wraps the PrimitivePacket into a network stream, enabling "Computational Liquidity."
pub struct NetworkActuatorDriver {
    pub id: String,
    pub remote_addr: String,
}

impl NetworkActuatorDriver {
    pub fn new(id: String, remote_addr: String) -> Self {
        Self { id, remote_addr }
    }

    /// Sends the primitive packet over TCP to the remote unia-node
    fn send_remote_request(&self, packet: &PrimitivePacket) -> Result<String, String> {
        let mut stream = TcpStream::connect(&self.remote_addr)
            .map_err(|e| format!("Failed to connect to remote resource {}: {}", self.id, e))?;
        
        // Serialize packet to JSON for network transport
        let payload = serde_json::to_string(packet)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        stream.write_all(payload.as_bytes())
            .map_err(|e| format!("Write error: {}", e))?;
        
        // Read response
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer)
            .map_err(|e| format!("Read error: {}", e))?;
        
        Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
    }
}

impl crate::nucleus::ActuatorDriver for NetworkActuatorDriver {
    fn get_resource_id(&self) -> String {
        self.id.clone()
    }

    fn execute(&self, packet: &PrimitivePacket, _state: &mut HashMap<String, HashMap<String, String>>) -> Result<String, String> {
        // Instead of local execution, we route to the remote address
        println!("[NetworkDriver] Routing primitive {:?} to remote address {}", packet.payload.primitive, self.remote_addr);
        self.send_remote_request(packet)
    }
}
