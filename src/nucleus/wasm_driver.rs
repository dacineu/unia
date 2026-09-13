use crate::bridge::primitive::{PrimitivePacket, UniversalPrimitive};
use std::collections::HashMap;

/// The WasmDriver allows a .ure resource to be implemented as a WebAssembly module.
/// This enables the same driver to run in a browser or on a server flawlessly.
pub struct WasmDriver {
    pub id: String,
    pub module_name: String,
    /// In a real implementation, this would hold the Wasm Instance (e.g., via Wasmer or Wasmtime)
    pub wasm_binary: Vec<u8>,
}

impl WasmDriver {
    pub fn new(id: String, module_name: String, binary: Vec<u8>) -> Self {
        Self {
            id,
            module_name,
            wasm_binary: binary,
        }
    }

    /// Simulates calling a Wasm exported function based on the Universal Primitive
    fn call_wasm_export(&self, primitive: &UniversalPrimitive, args: &HashMap<String, String>) -> Result<String, String> {
        // In a real Wasm implementation, we would:
        // 1. Look up the export name (e.g., "execute_reset")
        // 2. Write args to Wasm linear memory
        // 3. Call the function and read the result from memory
        
        match primitive {
            UniversalPrimitive::Reset => Ok(format!("WasmModule({}): executed RESET", self.module_name)),
            UniversalPrimitive::SetValue => Ok(format!("WasmModule({}): executed SET_VALUE", self.module_name)),
            UniversalPrimitive::GetValue => Ok(format!("WasmModule({}): executed GET_VALUE", self.module_name)),
            _ => Err(format!("Primitive {:?} not implemented in Wasm module {}", primitive, self.module_name)),
        }
    }
}

impl crate::nucleus::ActuatorDriver for WasmDriver {
    fn get_resource_id(&self) -> String {
        self.id.clone()
    }

    fn execute(&self, packet: &PrimitivePacket, state: &mut HashMap<String, HashMap<String, String>>) -> Result<String, String> {
        let resource_state = state.get_mut(&self.id).unwrap();
        
        // Call the Wasm internal logic
        let result = self.call_wasm_export(&packet.payload.primitive, &packet.payload.arguments)?;
        
        // Update the simulated state to maintain backward compatibility with the prototype
        if packet.payload.primitive == UniversalPrimitive::Reset {
            resource_state.insert("status".to_string(), "wasm_reset".to_string());
        }

        Ok(result)
    }
}
