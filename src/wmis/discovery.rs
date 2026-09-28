use crate::wmis::{SharingScope, WmisResource};
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
        println!(
            "🌐 WMIS: Broadcasting actuator {} to global fabric",
            resource.id
        );
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
                // **Tag check. This rejected every tagged query.**
                //
                // It read `if !query.tags.is_empty() { return false; }` with a
                // comment saying the real check had not been written yet, which
                // made the *unfinished* branch the *rejecting* one. Both production
                // callers — `FluidFactory::resolve_best_actuator` and
                // `OrchestratorMeta` — always send a tag, so both always got zero
                // results. The mesh was not merely unwired; it was unreachable, and
                // the only test that passed was the one that happened to send no
                // tags at all.
                //
                // Measured before the fix: three resources on the mesh, three
                // found with no tags, **zero** found with a tag. After: a resource
                // matches when it declares every requested capability.
                if !query
                    .tags
                    .iter()
                    .all(|tag| res.capabilities.iter().any(|c| c == tag))
                {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }
}
