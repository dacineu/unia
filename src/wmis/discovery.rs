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

    /// Withdraws an offering from the mesh.
    ///
    /// **Every offering so far was permanent.** `WmisResource` has carried an
    /// `owner` field since the type was written and nothing ever read it, so the
    /// party that could have withdrawn an offering was recorded and never asked --
    /// and no party could, because there was no way to.
    ///
    /// A contract whose terms say it may be exited "choosably definitive from all
    /// contractual parties" needs one thing first: the ability to leave. An offering
    /// that cannot be retracted is not a contract, it is a grant, and the difference
    /// is the whole of what makes the other party a party.
    ///
    /// Unilateral and unauthorised on purpose, which is what "from all contractual
    /// parties" means: there is no permission to obtain and no counterparty to ask.
    /// Returns whether anything was there to withdraw, so a caller can tell a
    /// withdrawal from a no-op -- an event that leaves no trace is
    /// indistinguishable from one that never happened.
    pub fn withdraw(&self, id: &str) -> bool {
        let mut mesh = self.mesh.write().unwrap();
        mesh.remove(id).is_some()
    }

    /// Every offering currently on the mesh, whatever its scope or quality.
    pub fn published(&self) -> Vec<WmisResource> {
        self.mesh.read().unwrap().values().cloned().collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wmis::adapter::{QualityMetrics, ResourceType, WmisResource};
    use std::collections::BTreeSet;

    fn offering(id: &str, capability: &str) -> WmisResource {
        WmisResource {
            id: id.to_string(),
            resource_type: ResourceType::Application,
            owner: "someone".into(),
            sharing_scope: SharingScope::Global,
            capabilities: vec![capability.to_string()],
            quality: QualityMetrics {
                qor: 0.9,
                qos: 0.9,
                qop: 0.9,
            },
            metadata: serde_json::json!({}),
        }
    }

    fn query(tags: &[&str]) -> DiscoveryQuery {
        DiscoveryQuery {
            seeker: "test".into(),
            allowed_scopes: vec![SharingScope::Global],
            min_qor: 0.0,
            tags: tags.iter().map(|t| t.to_string()).collect(),
        }
    }

    /// A contract that may be exited "from all contractual parties" needs the
    /// ability to leave, and there was none: `WmisResource` has carried an `owner`
    /// field since the type was written and nothing ever read it, so the party that
    /// could have withdrawn an offering was recorded and never asked.
    #[test]
    fn an_offering_can_be_withdrawn_and_stops_being_discoverable() {
        let mesh = WmisDiscoveryProvider::new();
        mesh.broadcast_actuator(offering("act-1", "reasoning"));
        assert_eq!(mesh.discover_resources(query(&["reasoning"])).len(), 1);

        assert!(
            mesh.withdraw("act-1"),
            "the withdrawal reported nothing was there"
        );
        assert_eq!(
            mesh.discover_resources(query(&["reasoning"])).len(),
            0,
            "a withdrawn offering is still discoverable, so leaving is not possible"
        );
    }

    /// Unilateral and unauthorised — "from all contractual parties" means there is
    /// no permission to obtain and no counterparty to ask. `withdraw` takes no
    /// identity and consults no owner, and the test is that a caller needs neither.
    #[test]
    fn withdrawal_needs_no_permission_and_no_counterparty() {
        let mesh = WmisDiscoveryProvider::new();
        mesh.broadcast_actuator(offering("act-1", "reasoning"));
        mesh.broadcast_actuator(offering("act-2", "interface"));

        // No owner, no token, no counterparty signature -- and the offering leaves.
        assert!(mesh.withdraw("act-1"));
        assert_eq!(mesh.published().len(), 1, "both offerings went");
        assert_eq!(mesh.published()[0].id, "act-2", "the wrong one went");
    }

    /// A withdrawal of something absent is a no-op and says so, because an event
    /// that leaves no trace is indistinguishable from one that never happened.
    #[test]
    fn withdrawing_something_absent_reports_that_nothing_was_there() {
        let mesh = WmisDiscoveryProvider::new();
        assert!(
            !mesh.withdraw("never-existed"),
            "a withdrawal of nothing reported success, so a caller cannot tell the \\
             two apart"
        );
    }

    /// One party's withdrawal does not disturb another's. Offers are independent,
    /// so exiting a contract is unilateral in fact and not only in principle.
    #[test]
    fn one_party_withdrawing_leaves_the_others_published() {
        let mesh = WmisDiscoveryProvider::new();
        for (i, cap) in ["reasoning", "interface", "efficiency"].iter().enumerate() {
            mesh.broadcast_actuator(offering(&format!("act-{i}"), cap));
        }
        assert_eq!(mesh.published().len(), 3);

        mesh.withdraw("act-1");
        let remaining: BTreeSet<String> = mesh.published().into_iter().map(|r| r.id).collect();
        assert_eq!(
            remaining,
            ["act-0".to_string(), "act-2".to_string()]
                .into_iter()
                .collect(),
            "one withdrawal disturbed another party's offering"
        );
    }
}
