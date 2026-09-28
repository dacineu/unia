use crate::fluid::{MicroNucleus, SlotType};
use crate::registry::ActuatorRegistry;
use crate::wmis::{DiscoveryQuery, SharingScope, WmisDiscoveryProvider};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct SynthesisRequest {
    pub objective: String,
    pub required_capabilities: Vec<String>,
    /// The scope the requester is asking for.
    ///
    /// A *request*, not a grant. Whether it is granted is decided by
    /// [`SynthesisRequest::credit`] and nothing else — the old code widened this
    /// into `allowed_scopes` unconditionally, so asking for less got you more.
    pub preferred_scope: SharingScope,
    /// The quality floor a counterparty must meet, which is about *them*.
    pub min_qor: f64,
    /// What the requester has demonstrated, in `0.0..=1.0`.
    ///
    /// **This is what makes crossing a scope reversible.**
    ///
    /// A request used to be answered from the market's own gates, and the market
    /// could not say anything about the requester. So a creature's reach was set by
    /// the registry and its hash order rather than by anything it had done. Reach is
    /// now earned: below [`CREDIT_FLOOR`] a request is confined to
    /// [`SharingScope::Circle`], and at or above it every scope is reachable.
    ///
    /// The property that matters is the *round trip*. At full credit, going out to
    /// `Global` and back to `Circle` costs nothing and returns the same shape,
    /// because the credit held throughout. Below the floor the crossing is not
    /// merely refused — the wider shape is *lost*, and asking again does not recover
    /// it, because nothing in the system re-earns credit on a creature's behalf.
    ///
    /// This is the same asymmetry the creature's own economy has: power that is
    /// demonstrated rather than owned, so it can be spent by neglect and it can be
    /// re-earned only by production. A scope is a promise about what a creature can
    /// still do, and a promise nobody has demonstrated is not one.
    pub credit: f64,
}

/// The credit below which a request is confined to its own circle.
pub const CREDIT_FLOOR: f64 = 0.2;

/// The scopes a given amount of demonstrated credit can reach.
///
/// Not a widening and not a preference: a *reachability* test. Below the floor a
/// creature can address its own circle and nothing beyond it, however global the
/// question it is asking. At or above the floor it can address any scope and the
/// crossing is reversible while the credit holds.
pub fn reachable_scopes(credit: f64) -> Vec<SharingScope> {
    use SharingScope::*;
    if credit >= CREDIT_FLOOR {
        vec![Circle, User, Team, Region, Country, Continent, Global]
    } else {
        vec![Circle]
    }
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

        // 2. Search WMIS Mesh, bounded by what the requester has demonstrated.
        //
        // **The scope gate used to widen.** It was built as
        // `[preferred_scope, Global]`, so a `Circle` request saw `Circle ∪ Global`
        // and a `Global` request saw only `Global` — asking for less got you more,
        // and a `Global` request was refused outright by a qor-0.9 offering sitting
        // in its own circle. Measured; see `shape_tests`.
        //
        // Now reach is earned. `reachable_scopes` is a function of demonstrated
        // credit alone, so a requester with credit can cross in either direction
        // and keeps its shape, and one without is confined to its circle however
        // global the question is.
        let reachable = reachable_scopes(request.credit);
        if !reachable.contains(&request.preferred_scope) {
            return Err(format!(
                "Cannot reach {:?} on {} demonstrated credit: reachable is {:?}, \
                 and the floor is {CREDIT_FLOOR}",
                request.preferred_scope, request.credit, reachable
            )
            .into());
        }
        let query = DiscoveryQuery {
            seeker: "FluidFactory".to_string(),
            allowed_scopes: reachable,
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
        ask_with_credit(f, scope, min_qor, 1.0)
    }

    /// The same request at a given amount of demonstrated credit.
    fn ask_with_credit(
        f: &FluidFactory,
        scope: SharingScope,
        min_qor: f64,
        credit: f64,
    ) -> Result<MicroNucleus, String> {
        f.synthesize_nucleus(SynthesisRequest {
            objective: "a creature that can feed itself".into(),
            required_capabilities: Vec::new(),
            preferred_scope: scope,
            min_qor,
            credit,
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

    /// **Defect: nothing ranks the candidates, so the winner is arbitrary.**
    ///
    /// This test previously asserted that the *qor-0.2* offering won the `Logic`
    /// slot, on the reading that quality was being ignored. That was not evidence of
    /// anything. Nothing ranks, `discover_resources` returns a `Vec` in `HashMap`
    /// order, and the caller takes the first -- so which offering wins is decided by
    /// hash order, and it differs between two markets. With the scope gate fixed the
    /// qor-0.9 offering became visible and won instead, which is the same arbitrary
    /// outcome wearing a different id.
    ///
    /// So the defect is not "the worse one wins". It is that **there is no way to
    /// know in advance which one wins**, when the two candidates differ only in
    /// quality -- the one thing that ought to decide it. That is what is asserted:
    /// the slot is filled by one of the two offerings that declare `reasoning`, and
    /// nothing in the code would have chosen the higher-quality one.
    #[test]
    fn nothing_ranks_the_candidates_so_the_winner_is_not_predictable() {
        let (mesh, _) = market();
        let mut winners: Vec<String> = (0..12)
            .filter_map(|_| {
                ask_with_credit(&factory(Arc::clone(&mesh)), SharingScope::Global, 0.0, 1.0)
                    .ok()
                    .and_then(|n| {
                        n.slots
                            .iter()
                            .find(|(k, _)| **k == SlotType::Logic)
                            .map(|(_, v)| v.to_string())
                    })
            })
            .collect();
        let distinct: std::collections::BTreeSet<String> = winners.iter().cloned().collect();

        assert!(
            !distinct.is_empty(),
            "the market resolved nothing, so the absence of a ranking is untested"
        );
        let offered = [
            "11111111-1111-1111-1111-111111111111",
            "22222222-2222-2222-2222-222222222222",
        ];
        for w in distinct.iter() {
            assert!(
                offered.contains(&w.as_str()),
                "the Logic slot was filled by {w}, which offers no `reasoning` at all"
            );
        }
        winners.dedup();
    }

    /// **Reach is earned, and a request is a request.**
    ///
    /// This replaces a test that asserted the opposite inversion, written to pin
    /// the old behaviour: `allowed_scopes` was `[preferred_scope, Global]`, so a
    /// `Circle` request saw `Circle ∪ Global` and a `Global` request saw only
    /// `Global`. Measured, asking for less got you more.
    ///
    /// `allowed_scopes` is now a function of demonstrated credit and nothing else,
    /// so a requester is served within its reach and refused outside it. The scope a
    /// request *asks for* no longer silently widens what it is granted.
    #[test]
    fn reach_is_earned_rather_than_widened_by_asking_for_less() {
        assert_eq!(
            reachable_scopes(0.0),
            vec![SharingScope::Circle],
            "a requester with no demonstrated credit can reach beyond its own circle"
        );
        assert_eq!(
            reachable_scopes(CREDIT_FLOOR - 0.01),
            vec![SharingScope::Circle],
            "a hair under the floor reached the globe"
        );
        assert_eq!(
            reachable_scopes(CREDIT_FLOOR),
            vec![
                SharingScope::Circle,
                SharingScope::User,
                SharingScope::Team,
                SharingScope::Region,
                SharingScope::Country,
                SharingScope::Continent,
                SharingScope::Global,
            ],
            "the floor does not open every scope at once"
        );
    }

    /// A request beyond the requester's reach is refused **for that reason**, and
    /// the refusal says so rather than blaming the market.
    #[test]
    fn a_global_request_is_refused_on_credit_and_not_on_the_market() {
        // Every offering at `Circle`, because a requester with no demonstrated
        // credit can reach only its own circle and would find nothing otherwise.
        // That isolation is the rule working rather than failing: a requester that
        // has demonstrated nothing has no standing anywhere. The first version of
        // this fixture published everything at `Global`, and the positive case
        // failed, which read as the credit gate being too strict when it was the
        // fixture that was wrong.
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
                SharingScope::Circle,
            ),
            offering(
                "44444444-4444-4444-4444-444444444444",
                "verification",
                0.9,
                SharingScope::Circle,
            ),
            offering(
                "55555555-5555-5555-5555-555555555555",
                "efficiency",
                0.9,
                SharingScope::Circle,
            ),
        ] {
            mesh.broadcast_actuator(r);
        }
        let f = factory(Arc::clone(&mesh));

        let Err(refused) = ask_with_credit(&f, SharingScope::Global, 0.0, 0.0) else {
            panic!("a requester with no credit was served a global shape")
        };
        assert!(
            refused.contains("demonstrated credit") && refused.contains("floor"),
            "the refusal does not name the credit or the floor, so it reads as the \
             market being empty rather than as the requester being unreached: {refused}"
        );
        assert!(
            ask_with_credit(&f, SharingScope::Circle, 0.0, 0.0).is_ok(),
            "the same requester within its own circle was refused, so the gate is not \
             about reach at all"
        );
    }

    /// **The round trip is free while the credit holds.** Out to `Global` and back
    /// to `Circle`, with the same shape either side.
    ///
    /// This is the sentence the whole rule exists for: a crossing is reversible
    /// *provided the credit is known*, so neither the direction nor the shape is
    /// lost by having been away. Both the outward and the return leg are asserted,
    /// because a rule that only lets you leave is a trap and a rule that only lets
    /// you come back is a cage.
    #[test]
    fn the_crossing_is_reversible_while_the_credit_holds() {
        let (mesh, _) = market();
        let f = factory(Arc::clone(&mesh));
        let shape_of = |scope| {
            let n = ask_with_credit(&f, scope, 0.0, 1.0).unwrap_or_else(|e| {
                panic!(
                    "{:?} should be reachable at full credit: {e}",
                    scope.clone()
                )
            });
            let mut v: Vec<String> = n
                .slots
                .iter()
                .map(|(k, val)| format!("{k:?}={val}"))
                .collect();
            v.sort();
            v.join(" ")
        };

        let before = shape_of(SharingScope::Circle);
        let outward = shape_of(SharingScope::Global);
        let back = shape_of(SharingScope::Circle);

        assert_eq!(
            before, back,
            "coming back returned a different shape than left"
        );
        assert_eq!(
            outward, back,
            "the outward leg and the return leg disagree, so the crossing changed \
             something it should not have"
        );
    }

    /// And crossing *without* credit is not merely refused: the wider shape is lost
    /// and asking again does not recover it, because nothing in the system
    /// re-earns credit on a creature's behalf.
    #[test]
    fn crossing_without_credit_does_not_recover_by_asking_again() {
        let (mesh, _) = market();
        let f = factory(mesh);
        // Matched rather than `expect_err`, which would need `MicroNucleus: Debug`.
        let (first, second) = match (
            ask_with_credit(&f, SharingScope::Global, 0.0, 0.0),
            ask_with_credit(&f, SharingScope::Global, 0.0, 0.0),
        ) {
            (Err(a), Err(b)) => (a, b),
            (Ok(_), _) => panic!("a requester with no credit was served a global shape"),
            (_, Ok(_)) => panic!(
                "asking again recovered the shape, so the wider shape was refused \
                 rather than lost"
            ),
        };
        assert_eq!(
            first, second,
            "the second attempt failed differently from the first, so something \
             changed between them"
        );
    }

    /// The scope-and-quality trap is gone, because it *was* the inversion.
    ///
    /// A `Global` request at `min_qor 0.5` used to refuse a qor-0.9 offering that
    /// was sitting in its own circle, because the scope gate excluded `Circle` from
    /// a `Global` query and the quality floor then had nothing left. With reach a
    /// function of credit, a requester with credit can see its own circle, so the
    /// good offering is available and the floor is honoured on its own terms.
    #[test]
    fn a_quality_floor_is_honoured_without_the_scope_gating_the_good_offering_out() {
        let (mesh, _) = market();
        let f = factory(mesh);
        assert!(
            ask_with_credit(&f, SharingScope::Global, 0.0, 1.0).is_ok(),
            "with no floor the Global request is served"
        );
        // The qor-0.9 `Circle` offering of `reasoning` is now visible, so a floor of
        // 0.5 is met by it rather than by the qor-0.2 Global one.
        assert!(
            ask_with_credit(&f, SharingScope::Global, 0.5, 1.0).is_ok(),
            "a qor-0.5 floor refused a qor-0.9 offering, so the scope gate is still \
             hiding the requester's own circle from it"
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
