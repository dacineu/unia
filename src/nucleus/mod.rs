use crate::bridge::primitive::{PrimitivePacket, UniversalPrimitive};
use crate::wmis::{WmisEconomicLayer, WmisResource, WmisOperation};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The Actuator Nucleus is the final execution layer.
/// It maps Universal Primitives to actual hardware drivers.
pub struct ActuatorNucleus {
    /// Maps Resource IDs to their specific hardware driver implementation
    drivers: HashMap<String, Box<dyn ActuatorDriver + Send + Sync>>,
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
            state_store: Arc::new(Mutex::new(HashMap::new())),
            economy,
        }
    }

    pub fn register_driver(&mut self, driver: Box<dyn ActuatorDriver + Send + Sync>) {
        self.drivers.insert(driver.get_resource_id(), driver);
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
