use crate::wmis::{WmisResource, SharingScope};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct DiscoveryQuery {
    pub seeker: String,
    pub allowed_scopes: Vec<SharingScope>,
    pub min_qor: f64,
    pub tags: Vec<String>,
}

pub struct WmisDiscoveryProvider {
    // Thread-safe global resource mesh simulation
    mesh: Arc<RwLock<HashMap<String, WmisResource>>>,
}

impl WmisDiscoveryProvider {
    pub fn new() -> Self {
        Self {
            mesh: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn broadcast_actuator(&self, resource: WmisResource) {
        let mut mesh = self.mesh.write().unwrap();
        println!("🌐 WMIS: Broadcasting actuator {} to global fabric", resource.id);
        mesh.insert(resource.id.clone(), resource);
    }

    pub fn discover_resources(&self, query: DiscoveryQuery) -> Vec<WmisResource> {
        let mesh = self.mesh.read().unwrap();
        
        mesh.values()
            .filter(|res| {
                // Scope check
                if !query.allowed_scopes.contains(&res.sharing_scope) {
                    return false;
                }
                // Quality check
                if res.quality.qor < query.min_qor {
                    return false;
                }
                // Tags check (simplified)
                if !query.tags.is_empty() {
                    // In a real system, we'd check tags here
                    return false; 
                }
                true
            })
            .cloned()
            .collect()
    }
}
