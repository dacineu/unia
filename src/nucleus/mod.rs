pub mod upa_dispatcher;

use crate::bridge::primitive::{PrimitivePacket, UniversalPrimitive};
use crate::bridge::upa::{UpaPacket, UpaOp};
use crate::wmis::{WmisEconomicLayer, WmisResource, WmisOperation};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The Actuator Nucleus is the final execution layer.
/// It maps Universal Primitives to actual hardware drivers.
pub struct ActuatorNucleus {
    /// Maps Resource IDs to their specific hardware driver implementation
    drivers: HashMap<String, Box<dyn ActuatorDriver + Send + Sync>>,
    /// The UPA Dispatcher for managing virtualized computation
    pub upa_dispatcher: Arc<Mutex<crate::nucleus::upa_dispatcher::UpaDispatcher>>,
    /// Tracks the current simulated state of all resources for verification
    state_store: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    /// Integrated Economic Layer for pay-per-primitive actuation
    pub economy: Arc<Mutex<WmisEconomicLayer>>,
}

/// Interface for a hardware-specific driver
pub trait ActuatorDriver {
    fn execute(&self, packet: &PrimitivePacket, state: &mut HashMap<String, HashMap<String, String>>) -> Result<String, String>;
    fn get_resource_id(&self) -> String;
}

impl ActuatorNucleus {
    pub fn new(economy: Arc<Mutex<WmisEconomicLayer>>) -> Self {
        Self {
            drivers: HashMap::new(),
            upa_dispatcher: Arc::new(Mutex::new(crate::nucleus::upa_dispatcher::UpaDispatcher::new())),
            state_store: Arc::new(Mutex::new(HashMap::new())),
            economy,
        }
    }

    pub fn register_driver(&mut self, driver: Box<dyn ActuatorDriver + Send + Sync>) {
        self.drivers.insert(driver.get_resource_id(), driver);
    }

    /// Dispatch a UPA-Assembly packet via the UPA Dispatcher for virtualized compute
    pub fn dispatch_upa(&self, slot: &str, packet: UpaPacket, user: &str, resource_meta: &WmisResource) -> Result<Vec<String>, String> {
        // 1. ECONOMIC GATE
        {
            let mut econ = self.economy.lock().unwrap();
            econ.charge_actuation(user, resource_meta, &WmisOperation::Execute)?;
        }

        // 2. UPA ROUTING
        let targets = self.upa_dispatcher.lock().unwrap().route(slot, &packet);
        
        // 3. MULTI-TARGET EXECUTION
        let mut results = Vec::new();
        let mut state = self.state_store.lock().unwrap();

        for target_id in targets {
            let driver = self.drivers.get(&target_id)
                .ok_or_else(|| format!("No driver registered for UPA target {}", target_id))?;
            
            state.entry(target_id.clone()).or_insert_with(HashMap::new);
            
            // Convert UpaPacket to PrimitivePacket for the driver
            let prim_packet = self.convert_upa_to_primitive(&packet, &target_id);
            let res = driver.execute(&prim_packet, &mut state)?;
            results.push(res);
        }

        Ok(results)
    }

    fn convert_upa_to_primitive(&self, upa: &UpaPacket, target_id: &str) -> PrimitivePacket {
        let primitive = match &upa.op {
            UpaOp::Suma { .. } | UpaOp::Product { .. } => UniversalPrimitive::SetValue,
            UpaOp::Transform { .. } => UniversalPrimitive::Transform,
            UpaOp::Superposition { .. } | UpaOp::Entangle { .. } => UniversalPrimitive::Pulse,
        };

        PrimitivePacket {
            header: crate::bridge::primitive::PacketHeader {
                timestamp: 1694430000,
                request_id: upa.request_id.clone(),
                priority: crate::bridge::primitive::Priority::High,
            },
            payload: crate::bridge::primitive::PacketPayload {
                primitive,
                resource_id: target_id.to_string(),
                arguments: HashMap::new(), // In a real system, extract from UpaOp
            },
            context: crate::bridge::primitive::PacketContext {
                expected_state: None,
                timeout_ms: 100,
            },
        }
    }

    /// The core execution loop: Packet -> Economy -> Driver -> Hardware
    pub fn dispatch(&self, packet: PrimitivePacket, user: &str, resource_meta: &WmisResource) -> Result<String, String> {
        let resource_id = &packet.payload.resource_id;
        
        // 1. ECONOMIC GATE: Charge for the actuation
        {
            let mut econ = self.economy.lock().unwrap();
            econ.charge_actuation(user, resource_meta, &WmisOperation::Execute)?;
        }

        // 2. DRIVER LOOKUP
        let driver = self.drivers.get(resource_id)
            .ok_or_else(|| format!("No driver registered for resource {}", resource_id))?;

        let mut state = self.state_store.lock().unwrap();
        
        // Ensure the resource has a state entry
        state.entry(resource_id.clone()).or_insert_with(HashMap::new);

        // 3. EXECUTION
        let result = driver.execute(&packet, &mut state);
        
        match result {
            Ok(msg) => {
                println!("[Nucleus] SUCCESS: {}", msg);
                Ok(msg)
            },
            Err(e) => {
                println!("[Nucleus] ERROR: {}", e);
                Err(e)
            }
        }
    }
}

// --- Mock Drivers for Benchmarking ---

/// A mock driver for a Smart Valve
pub struct ValveDriver {
    pub id: String,
}

impl ActuatorDriver for ValveDriver {
    fn get_resource_id(&self) -> String {
        self.id.clone()
    }

    fn execute(&self, packet: &PrimitivePacket, state: &mut HashMap<String, HashMap<String, String>>) -> Result<String, String> {
        let resource_state = state.get_mut(&self.id).unwrap();

        match packet.payload.primitive {
            UniversalPrimitive::Reset => {
                resource_state.insert("flow_rate".to_string(), "0.0".to_string());
                resource_state.insert("status".to_string(), "closed".to_string());
                Ok(format!("Valve {} force-closed via Hardware GPIO Low", self.id))
            },
            UniversalPrimitive::SetValue => {
                let val = packet.payload.arguments.get("value")
                    .ok_or("Missing value argument")?;
                resource_state.insert("flow_rate".to_string(), val.clone());
                Ok(format!("Valve {} flow adjusted to {}", self.id, val))
            },
            _ => Err(format!("Primitive {:?} not supported by ValveDriver", packet.payload.primitive)),
        }
    }
}

/// A mock driver for a Temperature Sensor
pub struct TempSensorDriver {
    pub id: String,
}

impl ActuatorDriver for TempSensorDriver {
    fn get_resource_id(&self) -> String {
        self.id.clone()
    }

    fn execute(&self, packet: &PrimitivePacket, _state: &mut HashMap<String, HashMap<String, String>>) -> Result<String, String> {
        match packet.payload.primitive {
            UniversalPrimitive::GetValue => {
                Ok(format!("Sensor {} reading: 22.4C (I2C Read)", self.id))
            },
            _ => Err(format!("Primitive {:?} not supported by TempSensorDriver", packet.payload.primitive)),
        }
    }
}

/// A driver for a FileSystem resource (demonstrating hardware independence)
pub struct FileSystemDriver {
    pub id: String,
}

impl ActuatorDriver for FileSystemDriver {
    fn get_resource_id(&self) -> String {
        self.id.clone()
    }

    fn execute(&self, packet: &PrimitivePacket, state: &mut HashMap<String, HashMap<String, String>>) -> Result<String, String> {
        let resource_state = state.get_mut(&self.id).unwrap();

        match packet.payload.primitive {
            UniversalPrimitive::SetValue => {
                let content = packet.payload.arguments.get("content")
                    .ok_or("Missing 'content' argument for FS write")?;
                let path = packet.payload.arguments.get("path")
                    .ok_or("Missing 'path' argument for FS write")?;
                
                resource_state.insert("last_write".to_string(), path.clone());
                Ok(format!("FS Resource {}: wrote to {} (Simulated Syscall)", self.id, path))
            },
            UniversalPrimitive::GetValue => {
                let path = packet.payload.arguments.get("path")
                    .ok_or("Missing 'path' argument for FS read")?;
                Ok(format!("FS Resource {}: read from {} -> 'simulated_data'", self.id, path))
            },
            UniversalPrimitive::Reset => {
                Ok(format!("FS Resource {}: cleared cache/temp files", self.id))
            },
            _ => Err(format!("Primitive {:?} not supported by FileSystemDriver", packet.payload.primitive)),
        }
    }
}
