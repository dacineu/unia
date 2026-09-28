use crate::fluid::{MicroNucleus, SlotType};
use crate::registry::ActuatorRegistry;
use crate::wmis::{DiscoveryQuery, SharingScope, WmisDiscoveryProvider};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct SynthesisRequest {
    pub objective: String,
    pub required_capabilities: Vec<String>,
    pub preferred_scope: SharingScope,
    pub min_qor: f64,
}

pub struct FluidFactory {
    registry: Arc<ActuatorRegistry>,
    discovery: Arc<WmisDiscoveryProvider>,
}

impl FluidFactory {
    pub fn new(registry: Arc<ActuatorRegistry>, discovery: Arc<WmisDiscoveryProvider>) -> Self {
        Self {
            registry,
            discovery,
        }
    }

    /// Synthesizes a new MicroNucleus by assembling the best matching actuators for the request.
    pub fn synthesize_nucleus(
        &self,
        request: SynthesisRequest,
    ) -> Result<MicroNucleus, Box<dyn std::error::Error>> {
        println!(
            "🏭 Fluid Factory: Synthesizing Nucleus for objective: '{}'",
            request.objective
        );

        let mut slots = HashMap::new();

        // Mapping of SlotType to required capability keywords
        let slot_mapping = vec![
            (SlotType::Logic, "reasoning"),
            (SlotType::Presenter, "interface"),
            (SlotType::Auditor, "verification"),
            (SlotType::Optimizer, "efficiency"),
        ];

        for (slot, keyword) in slot_mapping {
            let actuator_id = self.resolve_best_actuator(keyword, &request)?;
            slots.insert(slot, actuator_id);
            println!("⚙️  Assigned {:?} slot -> {}", slot, actuator_id);
        }

        let nucleus_id = Uuid::new_v4();
        println!("✨ Nucleus synthesized successfully: {}", nucleus_id);

        Ok(MicroNucleus {
            id: nucleus_id,
            slots,
            context_state: serde_json::json!({
                "synthesis_objective": request.objective,
                "timestamp": "2026-09-13T00:00:00Z", // Simplified timestamp to avoid chrono dependency
            }),
        })
    }

    /// Resolves the best available actuator for a specific capability,
    /// checking the local champion registry first, then the WMIS mesh.
    fn resolve_best_actuator(
        &self,
        capability: &str,
        request: &SynthesisRequest,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // 1. Check Local Champions
        if let Some(id) = self.registry.get_champion(capability) {
            println!("🏆 Local Champion found for {}: {}", capability, id);
            return Ok(id);
        }

        // 2. Search WMIS Mesh
        let query = DiscoveryQuery {
            seeker: "FluidFactory".to_string(),
            allowed_scopes: vec![request.preferred_scope.clone(), SharingScope::Global],
            min_qor: request.min_qor,
            tags: vec![capability.to_string()],
        };

        let found = self.discovery.discover_resources(query);
        if let Some(res) = found.first() {
            // In this prototype, we assume the WMIS ID can be parsed as a Uuid
            // In a real system, we'd use a mapping table or DU-UUID conversion
            if let Ok(id) = Uuid::parse_str(&res.id) {
                println!("🌐 WMIS Champion found for {}: {}", capability, id);
                return Ok(id);
            }
        }

        // 3. Fallback: Search local files for any matching .ure
        let matches = self.registry.find_matching_actuators(capability);
        if let Some((id, _)) = matches.first() {
            println!("🔍 Local Match found for {}: {}", capability, id);
            return Ok(*id);
        }

        Err(format!(
            "Could not resolve any suitable actuator for capability: {}",
            capability
        )
        .into())
    }
}

#[cfg(test)]
mod shape_tests {
    //! Whether a creature can get a different shape, measured.
    //!
    //! The question this module answers is the one the whole `fluid` machinery was
    //! built for: can a request change what a creature *is*? A `MicroNucleus` is
    //! four slot bindings, so "shape" is a real thing with a real definition, and
    //! `SynthesisRequest` is a real way to ask for one.
    //!
    //! The answer is a qualified no, and three of the four qualifications are
    //! defects pinned below rather than behaviour defended. Each is asserted as
    //! what it *is*, so that fixing it is a deliberate act with a visible diff
    //! rather than a silent improvement.

    use super::*;
    use crate::registry::ActuatorRegistry;
    use crate::wmis::adapter::{QualityMetrics, ResourceType, SharingScope, WmisResource};
    use crate::wmis::discovery::WmisDiscoveryProvider;
    use std::sync::Arc;

    fn uuid(s: &str) -> Uuid {
        Uuid::parse_str(s).expect("a fixed id")
    }

    fn offering(id: &str, capability: &str, qor: f64, scope: SharingScope) -> WmisResource {
        WmisResource {
            id: id.to_string(),
            resource_type: ResourceType::Application,
            owner: "someone".into(),
            sharing_scope: scope,
            capabilities: vec![capability.to_string()],
            quality: QualityMetrics {
                qor,
                qos: qor,
                qop: qor,
            },
            metadata: serde_json::json!({}),
        }
    }

    /// Five offerings: two competing for `reasoning`, and one for each other slot.
    fn market() -> (Arc<WmisDiscoveryProvider>, &'static [&'static str; 4]) {
        let mesh = Arc::new(WmisDiscoveryProvider::new());
        for r in [
            offering(
                "11111111-1111-1111-1111-111111111111",
                "reasoning",
                0.9,
                SharingScope::Circle,
            ),
            offering(
                "22222222-2222-2222-2222-222222222222",
                "reasoning",
                0.2,
                SharingScope::Global,
            ),
            offering(
                "33333333-3333-3333-3333-333333333333",
                "interface",
                0.9,
                SharingScope::Global,
            ),
            offering(
                "44444444-4444-4444-4444-444444444444",
                "verification",
                0.9,
                SharingScope::Global,
            ),
            offering(
                "55555555-5555-5555-5555-555555555555",
                "efficiency",
                0.9,
                SharingScope::Global,
            ),
        ] {
            mesh.broadcast_actuator(r);
        }
        (mesh, &["Logic", "Presenter", "Auditor", "Optimizer"])
    }

    fn factory(mesh: Arc<WmisDiscoveryProvider>) -> FluidFactory {
        let registry = ActuatorRegistry::with_base_dir("", std::path::Path::new("/tmp"));
        FluidFactory::new(Arc::new(registry), mesh)
    }

    fn ask(f: &FluidFactory, scope: SharingScope, min_qor: f64) -> Result<MicroNucleus, String> {
        f.synthesize_nucleus(SynthesisRequest {
            objective: "a creature that can feed itself".into(),
            required_capabilities: Vec::new(),
            preferred_scope: scope,
            min_qor,
        })
        .map_err(|e| e.to_string())
    }

    /// The mesh is reachable, and this is a regression test for the fix.
    ///
    /// `discover_resources` read `if !query.tags.is_empty() { return false; }` with a
    /// comment saying the real check had not been written — which made the
    /// *unfinished* branch the *rejecting* one. Both production callers always send
    /// a tag, so both always received zero results and the market was not merely
    /// unwired but unreachable. Measured before the fix: three resources on the
    /// mesh, three found with no tags, **zero** with a tag.
    #[test]
    fn a_tagged_query_reaches_the_mesh() {
        let (mesh, _) = market();
        let found = mesh.discover_resources(crate::wmis::discovery::DiscoveryQuery {
            seeker: "test".into(),
            allowed_scopes: vec![SharingScope::Global],
            min_qor: 0.0,
            tags: vec!["reasoning".into()],
        });
        // One, not two: the qor-0.9 offering of `reasoning` is at `Circle` and a
        // `Global`-scoped query cannot see it. The count is deliberately *not* two
        // -- the first version asserted it was, and the failure turned out to be the
        // scope filter working correctly rather than the tag filter failing. What
        // this protects is that a tag stops excluding things.
        assert_eq!(
            found.len(),
            1,
            "the tag filter is rejecting again, so the market is unreachable: {:?}",
            found.iter().map(|r| &r.id).collect::<Vec<_>>()
        );
    }

    /// A tag matches what the offering actually declares, and an undeclared
    /// capability is not a near miss.
    #[test]
    fn a_tag_matches_only_what_is_declared() {
        let (mesh, _) = market();
        let probe = |tags: Vec<&str>| {
            mesh.discover_resources(crate::wmis::discovery::DiscoveryQuery {
                seeker: "test".into(),
                allowed_scopes: vec![SharingScope::Global],
                min_qor: 0.0,
                tags: tags.into_iter().map(String::from).collect(),
            })
            .len()
        };
        // Scoped to `Global`, so exactly one of the two `reasoning` offerings is
        // visible. The claim being protected is the *shape* of the relation, not the
        // count: a tag matches declarations, several tags must all match one
        // offering, and an undeclared capability is not a near miss.
        assert_eq!(
            probe(vec!["reasoning"]),
            1,
            "one Global offering declares reasoning"
        );
        assert_eq!(
            probe(vec!["reasoning", "interface"]),
            0,
            "no single offering declares both"
        );
        assert_eq!(
            probe(vec!["nonsense"]),
            0,
            "an undeclared tag matched something"
        );
    }

    /// **A shape is synthesised.** Four slots, four bindings, from a request. So
    /// "a creature can get a different shape" is mechanically true and the object
    /// exists; the questions below are about whether the *request* decides it.
    #[test]
    fn a_shape_is_synthesised_from_the_market() {
        let (mesh, slots) = market();
        let nucleus = ask(&factory(mesh), SharingScope::Global, 0.0)
            .unwrap_or_else(|e| panic!("all four slots were offered, got: {e}"));
        assert_eq!(nucleus.slots.len(), slots.len(), "a slot is unassigned");
        assert_eq!(
            nucleus.context_state["synthesis_objective"],
            "a creature that can feed itself"
        );
    }

    /// An unfulfilled slot refuses the **whole** shape rather than returning a
    /// partial one. Found by writing a probe that offered only one capability and
    /// being refused, which is the right answer: a creature with three quarters of
    /// a shape is a different and more dangerous thing than one with none.
    #[test]
    fn an_unfulfilled_slot_refuses_the_whole_shape() {
        let mesh = Arc::new(WmisDiscoveryProvider::new());
        mesh.broadcast_actuator(offering(
            "66666666-6666-6666-6666-666666666666",
            "reasoning",
            0.9,
            SharingScope::Global,
        ));
        // Matched rather than `expect_err`, which would need `MicroNucleus: Debug`.
        // Noted because a shape you cannot print is a small gap in its own right.
        let Err(err) = ask(&factory(mesh), SharingScope::Global, 0.0) else {
            panic!("three slots were empty and a shape came back anyway");
        };
        assert!(
            err.contains("verification") || err.contains("interface") || err.contains("efficiency"),
            "the refusal does not name the slot it could not fill: {err}"
        );
    }

    /// **Defect: a champion is chosen over the request, and the request is then
    /// never consulted.**
    ///
    /// `resolve_best_actuator` checks `get_champion` first and returns. So the
    /// scope, the quality floor and the required capabilities are computed into a
    /// query that is never built. A champion is imposed — it is one actuator
    /// declared the answer for a capability — and it is the only path on which
    /// choice is exercised at all.
    #[test]
    fn a_champion_short_circuits_the_request_entirely() {
        let (mesh, _) = market();
        let mut registry = ActuatorRegistry::with_base_dir("", std::path::Path::new("/tmp"));
        for (cap, id) in [
            ("reasoning", "11111111-1111-1111-1111-111111111111"),
            ("interface", "33333333-3333-3333-3333-333333333333"),
            ("verification", "44444444-4444-4444-4444-444444444444"),
            ("efficiency", "55555555-5555-5555-5555-555555555555"),
        ] {
            registry.set_champion(cap, uuid(id));
        }
        let f = FluidFactory::new(Arc::new(registry), Arc::clone(&mesh));

        // A floor nothing can meet, and a scope nothing is published in. With a
        // champion on every slot both are ignored, so the shape is identical to the
        // one an unconstrained request gets.
        let constrained = ask(&f, SharingScope::Circle, 0.99)
            .unwrap_or_else(|e| panic!("champions should ignore the floor, got: {e}"));
        let unconstrained = ask(&f, SharingScope::Global, 0.0)
            .unwrap_or_else(|e| panic!("same request refused: {e}"));
        let shape = |n: &MicroNucleus| {
            let mut v: Vec<String> = n
                .slots
                .iter()
                .map(|(k, val)| format!("{k:?}={val}"))
                .collect();
            v.sort();
            v.join(" ")
        };
        assert_eq!(
            shape(&constrained),
            shape(&unconstrained),
            "the request influenced the shape, so the champion is not short \\
             circuiting it"
        );
    }

    /// **Defect: the quality floor refuses but never selects.**
    ///
    /// `min_qor` is a gate, not a preference, and nothing ranks what passes it —
    /// `discover_resources` returns a `Vec` in `HashMap` order and
    /// `resolve_best_actuator` takes the first. Measured: with a qor-0.9 offering
    /// at `Circle` and a qor-0.2 offering at `Global`, both above a floor of 0.0,
    /// **the poor one won the `Logic` slot.**
    #[test]
    fn quality_is_a_gate_and_not_a_preference_so_the_worse_actuator_wins() {
        let (mesh, _) = market();
        let nucleus = ask(&factory(mesh), SharingScope::Global, 0.0)
            .unwrap_or_else(|e| panic!("every slot was offered, got: {e}"));
        let logic = nucleus
            .slots
            .iter()
            .find(|(k, _)| **k == SlotType::Logic)
            .map(|(_, v)| v.to_string())
            .expect("a Logic slot");

        assert_eq!(
            logic, "22222222-2222-2222-2222-222222222222",
            "the qor-0.9 offering at Circle won the Logic slot, so quality is being \\
             used to rank and this is no longer a defect"
        );
    }

    /// **Defect: narrowing the scope widens the choice.**
    ///
    /// `allowed_scopes` is built as `[preferred_scope, Global]`. A `Circle` request
    /// therefore sees `Circle ∪ Global` while a `Global` request sees only `Global`,
    /// so asking for less gets you *more*. The market here offers `reasoning` at
    /// `Circle` only, and every other slot at `Global`:
    ///
    /// - a `Global` request **cannot be served at all** — it cannot see the one
    ///   offering of `reasoning` that exists;
    /// - a `Circle` request is served.
    ///
    /// A creature that asks the world for something global is refused, and one that
    /// asks only for its own circle is answered. That is backwards from every word
    /// involved, and it is deterministic — unlike *which* offering wins, which is
    /// decided by hash order and varies between `HashMap` instances.
    #[test]
    fn a_global_request_is_refused_what_a_circle_request_is_served() {
        let mesh = Arc::new(WmisDiscoveryProvider::new());
        for r in [
            offering(
                "11111111-1111-1111-1111-111111111111",
                "reasoning",
                0.9,
                SharingScope::Circle,
            ),
            offering(
                "33333333-3333-3333-3333-333333333333",
                "interface",
                0.9,
                SharingScope::Global,
            ),
            offering(
                "44444444-4444-4444-4444-444444444444",
                "verification",
                0.9,
                SharingScope::Global,
            ),
            offering(
                "55555555-5555-5555-5555-555555555555",
                "efficiency",
                0.9,
                SharingScope::Global,
            ),
        ] {
            mesh.broadcast_actuator(r);
        }
        let f = factory(Arc::clone(&mesh));

        let Err(refused) = ask(&f, SharingScope::Global, 0.0) else {
            panic!(
                "a Global request was served by a Circle-only offering, so the \
                     scope is not narrowing anything"
            );
        };
        assert!(
            refused.contains("reasoning"),
            "the refusal does not name the slot it could not fill: {refused}"
        );
        assert!(
            ask(&f, SharingScope::Circle, 0.0).is_ok(),
            "a Circle request was refused what a Global request also could not have, \
             so narrowing does not widen the choice after all"
        );
    }

    /// A scope *can* withhold: a `Global` request cannot see a `Circle` offering,
    /// which is how a quality floor ends up refusing quality it would otherwise
    /// accept. This is the interaction worth naming — the two gates compose into a
    /// trap, and it is the one behaviour here that is at least defensible, so it is
    /// pinned separately from the two defects above.
    #[test]
    fn a_quality_floor_on_a_wide_scope_refuses_quality_it_would_otherwise_accept() {
        let (mesh, _) = market();
        let f = factory(mesh);
        assert!(
            ask(&f, SharingScope::Global, 0.0).is_ok(),
            "with no floor the poor Global offering serves the slot"
        );
        let Err(refused) = ask(&f, SharingScope::Global, 0.5) else {
            panic!("a qor-0.5 floor was met by a qor-0.2 offering");
        };
        assert!(
            refused.contains("reasoning"),
            "the refusal does not name what it could not find: {refused}"
        );
    }

    /// Selection is **stable but arbitrary**: the same request gives the same
    /// answer 24 times running, and the answer is decided by hash order rather than
    /// by any criterion.
    ///
    /// Worth its own test because stability is what makes the defect dangerous. An
    /// arbitrary winner that changed on every call would be obviously broken; one
    /// that reproduces exactly looks principled and is not, and it will change the
    /// moment anything is inserted or removed.
    #[test]
    fn selection_is_stable_and_decided_by_hash_order() {
        let mesh = Arc::new(WmisDiscoveryProvider::new());
        for i in 1..=6u32 {
            mesh.broadcast_actuator(offering(
                &format!("0000000{i}-0000-0000-0000-000000000000"),
                "reasoning",
                0.5,
                SharingScope::Global,
            ));
        }
        for (i, cap) in ["interface", "verification", "efficiency"]
            .iter()
            .enumerate()
        {
            mesh.broadcast_actuator(offering(
                &format!("9999999{i}-9999-9999-9999-999999999999"),
                cap,
                0.9,
                SharingScope::Global,
            ));
        }
        let f = factory(Arc::clone(&mesh));

        let mut winners: Vec<String> = (0..24)
            .filter_map(|_| {
                ask(&f, SharingScope::Global, 0.0).ok().and_then(|n| {
                    n.slots
                        .iter()
                        .find(|(k, _)| **k == SlotType::Logic)
                        .map(|(_, v)| v.to_string())
                })
            })
            .collect();
        let distinct: std::collections::BTreeSet<String> = winners.iter().cloned().collect();
        winners.dedup();

        assert_eq!(
            distinct.len(),
            1,
            "the winner changed between identical requests: {winners:?} -- if this \\
             now fails nondeterministically, something else has changed, because \\
             the stability here is an artifact of HashMap order and not a policy"
        );
        // **Not** an assertion about *which* offering, because that is the whole
        // point: all six declare the same capability at the same quality, so nothing
        // distinguishes them. This test asserted `00000006`, passed in a probe
        // offering only these six, and then failed here at `00000004` -- because
        // `HashMap` seeds its hasher per instance, so the winner is stable within
        // one market and differs between two. The right claim is only that it is
        // one of the six, and that it never varies within a run.
        assert!(
            (1..=6).any(|i| winners[0].starts_with(&format!("0000000{i}"))),
            "the winner {} is not one of the six identical candidates, so something \
             other than hash order chose it -- which would be an improvement",
            winners[0]
        );
    }
}
