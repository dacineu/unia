use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::bridge::primitive::{PrimitiveBridge, PrimitivePacket, UniversalPrimitive, PacketHeader, PacketPayload, PacketContext, Priority};
use crate::nucleus::ActuatorNucleus;
use crate::wmis::WmisEconomicLayer;

/// Syscall types emulated by the unia-OS kernel
#[derive(Debug, Clone, PartialEq)]
pub enum UniaSyscall {
    ReadDevice(String),
    WriteDevice(String, String),
    ExecuteCompute(String, Vec<String>),
    MountResource(String),
}

/// The Virtual Kernel acts as the bridge between a simulated OS userspace 
/// and the unia Actuator Nucleus.
pub struct UniaKernel {
    bridge: Arc<PrimitiveBridge>,
    nucleus: Arc<ActuatorNucleus>,
    vfs: Arc<Mutex<HashMap<String, String>>>, // Simulated Virtual File System
}

impl UniaKernel {
    pub fn new(bridge: Arc<PrimitiveBridge>, nucleus: Arc<ActuatorNucleus>) -> Self {
        Self {
            bridge,
            nucleus,
            vfs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Intercepts a virtual syscall and transforms it into a unia Primitive Packet
    pub fn handle_syscall(&self, syscall: UniaSyscall, user: &str) -> Result<String, String> {
        match syscall {
            UniaSyscall::ReadDevice(dev_id) => {
                // Map 'read' syscall to GetState primitive
                let packet = self.create_packet(dev_id, UniversalPrimitive::GetState);
                self.dispatch_to_nucleus(packet, user)
            },
            UniaSyscall::WriteDevice(dev_id, value) => {
                // Map 'write' syscall to SetValue primitive
                let mut args = HashMap::new();
                args.insert("value".to_string(), value);
                let packet = self.create_packet_with_args(dev_id, UniversalPrimitive::SetValue, args);
                self.dispatch_to_nucleus(packet, user)
            },
            UniaSyscall::ExecuteCompute(op_id, params) => {
                // Map 'compute' syscall to UPA-specific primitives (e.g., Suma)
                let mut args = HashMap::new();
                args.insert("params".to_string(), params.join(","));
                let packet = self.create_packet_with_args(op_id, UniversalPrimitive::Transform, args);
                self.dispatch_to_nucleus(packet, user)
            },
            UniaSyscall::MountResource(res_id) => {
                let mut vfs = self.vfs.lock().unwrap();
                vfs.insert(format!("/dev/unia/{}", res_id), res_id.clone());
                Ok(format!("Resource {} mounted to /dev/unia/{}", res_id, res_id))
            }
        }
    }

    fn create_packet(&self, res_id: &str, primitive: UniversalPrimitive) -> PrimitivePacket {
        self.create_packet_with_args(res_id, primitive, HashMap::new())
    }

    fn create_packet_with_args(&self, res_id: &str, primitive: UniversalPrimitive, args: HashMap<String, String>) -> PrimitivePacket {
        PrimitivePacket {
            header: PacketHeader {
                timestamp: 1694430000,
                request_id: "sys-req".to_string(),
                priority: Priority::High,
            },
            payload: PacketPayload {
                primitive,
                resource_id: res_id.to_string(),
                arguments: args,
            },
            context: PacketContext {
                expected_state: None,
                timeout_ms: 100,
            },
        }
    }

    fn dispatch_to_nucleus(&self, packet: PrimitivePacket, user: &str) -> Result<String, String> {
        // In a real OS, we would lookup the WmisResource metadata from the VFS
        let meta = crate::wmis::WmisResource {
            id: packet.payload.resource_id.clone(),
            resource_type: crate::wmis::ResourceType::Virtual,
            owner: user.to_string(),
            sharing_scope: crate::wmis::SharingScope::Global,
            capabilities: vec![],
            quality: crate::wmis::QualityMetrics { qor: 1.0, qos: 1.0, qop: 1.0 },
            metadata: serde_json::json!({}),
        };

        self.nucleus.dispatch(packet, user, &meta)
    }
}
