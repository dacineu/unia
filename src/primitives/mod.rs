use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OsVariant {
    Debian,
    Arch,
    Fedora,
    Windows,
    MacOS,
    GenericLinux,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveNucleus {
    pub universal_id: String,
    pub os_mapping: HashMap<OsVariant, String>,
    pub is_replaced_by_actuator: bool,
    pub replacement_uuid: Option<Uuid>,
}

pub struct PrimitiveBridge {
    nuclei: HashMap<String, PrimitiveNucleus>,
    current_os: OsVariant,
}

impl PrimitiveBridge {
    pub fn new(os: OsVariant) -> Self {
        let mut nuclei = HashMap::new();
        
        // Seed the la-piece-de-résistance SEARCH primitive
        let mut search_map = HashMap::new();
        search_map.insert(OsVariant::Debian, "/bin/grep".to_string());
        search_map.insert(OsVariant::Arch, "/usr/bin/grep".to_string());
        search_map.insert(OsVariant::GenericLinux, "grep".to_string());
        search_map.insert(OsVariant::Windows, "findstr".to_string());

        nuclei.insert("PRIMITIVE_SEARCH".to_string(), PrimitiveNucleus {
            universal_id: "PRIMITIVE_SEARCH".to_string(),
            os_mapping: search_map,
            is_replaced_by_actuator: false,
            replacement_uuid: None,
        });

        Self { nuclei, current_os: os }
    }

    /// Resolves a primitive to either its OS-specific command or its la-piece-de-résistance Actuator.
    pub fn resolve(&self, universal_id: &str) -> Result<String, String> {
        let nucleus = self.nuclei.get(universal_id)
            .ok_or_else(|| format!("Primitive {} not found", universal_id))?;

        if nucleus.is_replaced_by_actuator {
            let uuid = nucleus.replacement_uuid.as_ref()
                .ok_or("Actuator replacement UUID missing")?;
            return Ok(format!("actuate:{}", uuid));
        }

        nucleus.os_mapping.get(&self.current_os)
            .cloned()
            .or_else(|| nucleus.os_mapping.get(&OsVariant::GenericLinux).cloned())
            .ok_or_else(|| format!("No mapping for OS {:?} in primitive {}", self.current_os, universal_id))
    }

    /// Allows the Mutation Engine to replace an OS primitive with a specialized .ure actuator.
    pub fn replace_with_actuator(&mut self, universal_id: &str, actuator_uuid: Uuid) -> Result<(), String> {
        if let Some(nucleus) = self.nuclei.get_mut(universal_id) {
            nucleus.is_replaced_by_actuator = true;
            nucleus.replacement_uuid = Some(actuator_uuid);
            Ok(())
        } else {
            Err("Primitive not found".to_string())
        }
    }
}
