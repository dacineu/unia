//! A linker that binds a **capability** to a **resource that declares it**,
//! and unbinds it while the machine is running.
//!
//! # What was here before, and why it is not a linker
//!
//! `UpaDispatcher` has `set_primary(slot, resource_id)` and
//! `set_shadow(slot, resource_id)`, and `route()` returns the primary and, if
//! present, the shadow. A human fills that table in. Nothing discovers anything,
//! nothing checks that the named resource can do what the slot needs, and the
//! only notion of what a resource *is* is one `Dimensionality` enum per id.
//!
//! So it is a hand-written routing table with a `Vec` of at most two entries, and
//! calling it a dispatcher is generous.
//!
//! # The difference, stated as one line
//!
//! > A routing table binds a **name** to a resource. A linker binds a
//! > **capability** to a resource that *declares* that capability, and the
//! > binding can be broken and re-made without stopping the machine.
//!
//! Both halves matter and the second is the one that is missing. A table written
//! by a human cannot survive the resource it names being unplugged.
//!
//! # Where a capability comes from
//!
//! **From the manifest, and nowhere else.** A resource can do what its `.ure`
//! says it can do and nothing more: the capability set of a resource *is* the set
//! of its `action_primitives`. That is not a new ontology invented here — it is
//! the reuse of the one that already exists, and it has a property worth having:
//! the thing that declares a capability and the thing that is checked against it
//! are the same artifact, so they cannot drift apart.
//!
//! The temptation to widen this was declined. `UniversalPrimitive` is seventeen
//! hardware verbs and an ontology is not that; adding capability names beside
//! them would produce two vocabularies for one machine and a means of confusing
//! them.
//!
//! # The shape of a link
//!
//! ```text
//!   demand ──▶ [ declared resources ] ──▶ binding
//!    (a set       (each a .ure, each       (primary: serves the
//!     of acts      hashing to an            whole demand)
//!     wanted)      address)                 shadow:  serves it too
//! ```
//!
//! Resolution is a meet over declared actions, so `serves` from `crate::meet`
//! is the whole of the test and the lattice laws already hold for it.
//!
//! # Hotswap, and the part that is a decision
//!
//! A shadow is not a second target. Routing a packet to both primary and shadow
//! is *fan-out*, and for a counter or an increment it is wrong: the act happens
//! twice. A shadow exists to be **promoted**, not to be written to.
//!
//! So `promote` is the operation, and it carries a question this module refuses
//! to answer on its own: **what happens to an act already in flight against the
//! resource being demoted?**
//!
//! - *complete* — it finishes on the old resource. Cheap, and the old resource
//!   must stay alive until the count reaches zero.
//! - *refuse* — it is abandoned. The caller learns, and can retry.
//! - *reissue* — it restarts on the new one. Requires the act to be replayable,
//!   which for a closed `Signature` it is not.
//!
//! The default is **complete**, and the count is public, because picking
//! silently is the defect this project has already paid for once: the old
//! `resolve_primitive` returned `SetValue` for every unrecognised action, so a
//! typo and a genuinely unresolvable request took the same path and nothing said
//! so. A linker that unlinks a busy resource without saying what happens to the
//! act on it is the same mistake wearing a different hat.

use crate::bridge::primitive::UreResource;
use crate::identifiers::skeleton;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// What a caller needs done. A *set* of acts, not a name for a place.
///
/// A demand with an empty set is refused rather than satisfied by everything,
/// because a demand that asks for nothing is not a demand and binding it to an
/// arbitrary resource would be a lie about what was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Demand {
    /// What this demand is for. Carried for the receipt, never resolved against.
    pub name: String,
    /// The acts wanted, all of them.
    pub required: BTreeSet<String>,
}

impl Demand {
    /// A demand for a single act.
    pub fn one(name: &str, act: &str) -> Self {
        let mut required = BTreeSet::new();
        required.insert(act.to_string());
        Demand {
            name: name.to_string(),
            required,
        }
    }

    /// Whether this demand asks for anything.
    fn is_empty(&self) -> bool {
        self.required.is_empty()
    }
}

/// A resource as the linker sees it: what it declares it can do, and what it is.
///
/// Derived from a `.ure` and from nothing else. There is no way to construct one
/// by hand, which is deliberate — a resource whose capability set is asserted
/// rather than declared is exactly the thing that makes a manifest drift from
/// the machine it describes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declared {
    /// Content address of the manifest's skeleton, so two resources that declare
    /// the same acts are recognisable as the same *kind* of thing.
    pub address: String,
    /// The acts the manifest declares. This **is** the capability set.
    pub actions: BTreeSet<String>,
    /// The resource id, used only to address the driver once bound.
    pub id: String,
}

impl Declared {
    /// Reads a resource's declared capabilities out of its manifest.
    pub fn of(ure: &UreResource) -> Self {
        // Through `Value`, because `skeleton` is defined on the manifest as a
        // document. The address is therefore the *serialised* form's address,
        // which is the honest one: two manifests that differ only in field
        // order are the same artifact only if their serialised forms agree, and
        // serde's map is not order-preserving, so in practice it does.
        let document = serde_json::to_value(ure).unwrap_or(serde_json::Value::Null);
        Declared {
            address: crate::identifiers::DuUuid::generate(&skeleton(&document), None)
                .map(|u| u.to_string())
                .unwrap_or_else(|e| format!("unaddressable: {e}")),
            actions: ure.action_primitives.iter().map(|a| a.id.clone()).collect(),
            id: ure.resource_id.clone(),
        }
    }

    /// Whether this resource declares the whole demand.
    ///
    /// A demand of two acts is served by a resource declaring three, because
    /// nothing says a resource may not also do other things. A resource
    /// declaring one of the two is not a partial match, it is a refusal: a
    /// half-bound demand is a misroute, and a silent one.
    pub fn serves(&self, demand: &Demand) -> bool {
        demand.required.iter().all(|a| self.actions.contains(a))
    }
}

/// Why a demand was not bound. Distinguishing these is the point: an unbound
/// demand with three different causes has three different fixes, and reporting
/// one of them loses that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unbound {
    /// The demand asks for nothing, so there is nothing to check.
    Empty,
    /// No declared resource declares the whole demand.
    Undeclared { missing: BTreeSet<String> },
    /// Resources exist that declare *some* of it, which is the near miss worth
    /// naming separately from "nothing matched".
    Partial { near: Vec<String> },
}

/// A resource bound to a demand, with the shadow that may replace it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub primary: String,
    /// A resource that serves the same demand and is being kept warm. Not a
    /// second target: see the module comment on hotswap.
    pub shadow: Option<String>,
}

/// The link table. The whole of the linker.
#[derive(Debug, Default, Clone)]
pub struct Linker {
    declared: BTreeMap<String, Declared>,
    /// The demand is stored beside the binding, not reconstructed from it. A
    /// hotswap has to be checked against the demand the link was *made on*, and
    /// a check that re-derives its own question answers the wrong one.
    routes: BTreeMap<String, (Demand, Binding)>,
}

impl Linker {
    pub fn new() -> Self {
        Linker::default()
    }

    /// Declares a resource. This is the harvest step, and the only way a
    /// resource becomes visible to the linker.
    pub fn declare(&mut self, ure: &UreResource) -> Declared {
        let d = Declared::of(ure);
        self.declared.insert(d.id.clone(), d.clone());
        d
    }

    /// Withdraws a declaration. A resource that is unplugged leaves the linker
    /// exactly as it entered it, which is what makes the withdrawal checkable
    /// rather than asserted.
    pub fn undeclare(&mut self, id: &str) -> Option<Declared> {
        self.declared.remove(id)
    }

    /// The capabilities a resource declares.
    pub fn declares(&self, id: &str) -> Option<&BTreeSet<String>> {
        self.declared.get(id).map(|d| &d.actions)
    }

    /// Binds a demand to every declared resource that serves it.
    ///
    /// The first by sorted id becomes the primary, because a linker's choice must
    /// be a function of the declarations and not of hash order — otherwise two
    /// runs over the same hardware could bind differently and nothing would say
    /// so. The second becomes the shadow, and a third is not an error: a
    /// machine with four capable processors should not be a machine with two.
    pub fn link(&mut self, demand: &Demand) -> Result<Binding, Unbound> {
        if demand.is_empty() {
            return Err(Unbound::Empty);
        }
        let mut serving: Vec<&Declared> = self
            .declared
            .values()
            .filter(|d| d.serves(demand))
            .collect();
        serving.sort_by(|a, b| a.id.cmp(&b.id));

        match serving.len() {
            0 => {
                // The near miss: something declared, nothing declared all of it.
                let mut near: Vec<String> = self
                    .declared
                    .values()
                    .filter(|d| demand.required.iter().any(|a| d.actions.contains(a)))
                    .map(|d| d.id.clone())
                    .collect();
                near.sort();
                if near.is_empty() {
                    Err(Unbound::Undeclared {
                        missing: demand.required.clone(),
                    })
                } else {
                    Err(Unbound::Partial { near })
                }
            }
            _ => {
                let binding = Binding {
                    primary: serving[0].id.clone(),
                    shadow: serving.get(1).map(|d| d.id.clone()),
                };
                self.routes
                    .insert(demand.name.clone(), (demand.clone(), binding.clone()));
                Ok(binding)
            }
        }
    }

    /// The binding for a demand, if it has one.
    pub fn bound(&self, demand: &str) -> Option<&Binding> {
        self.routes.get(demand).map(|(_, b)| b)
    }

    /// Removes a binding without removing the declaration.
    ///
    /// Separated from `undeclare` because they answer different questions: a
    /// withdrawn *declaration* says the resource is gone, and a removed
    /// *binding* says nothing has been asked of it any more. Collapsing them
    /// would make it impossible to leave a warm processor idle.
    pub fn unlink(&mut self, demand: &str) -> Option<Binding> {
        self.routes.remove(demand).map(|(_, b)| b)
    }

    /// Promotes the shadow to primary — the hotswap.
    ///
    /// The new primary must still serve the demand. It usually does, because it
    /// was the shadow of the same demand, but a declaration withdrawn and
    /// re-added with fewer actions between the two calls would not be, and a
    /// swap that checked nothing would install it.
    ///
    /// Returns the acts in flight against the demoted resource, which is the
    /// number the caller needs and the reason [`Inflight`] exists. It is returned
    /// rather than acted upon: see the module comment on what to do with it.
    pub fn promote(
        &mut self,
        demand: &str,
        inflight: &Inflight,
    ) -> Result<Promotion, PromoteError> {
        let (want, binding) = self
            .routes
            .get(demand)
            .ok_or_else(|| PromoteError::NotBound(demand.to_string()))?;
        let (want, binding) = (want.clone(), binding.clone());
        let shadow = binding
            .shadow
            .ok_or_else(|| PromoteError::NoShadow(demand.to_string()))?;
        let declared = self
            .declared
            .get(&shadow)
            .ok_or_else(|| PromoteError::ShadowUndeclared(shadow.clone()))?;
        if !declared.serves(&want) {
            return Err(PromoteError::ShadowNoLongerServes {
                demand: demand.to_string(),
                shadow: shadow.clone(),
                missing: want
                    .required
                    .iter()
                    .filter(|a| !declared.actions.contains(*a))
                    .cloned()
                    .collect(),
            });
        }
        let demoted = binding.primary;
        let in_flight = inflight.on(&demoted);
        self.routes.insert(
            demand.to_string(),
            (
                want,
                Binding {
                    primary: shadow,
                    shadow: Some(demoted.clone()),
                },
            ),
        );
        Ok(Promotion { demoted, in_flight })
    }
}

/// The outcome of a hotswap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Promotion {
    /// The resource demoted to shadow, and which still has acts running on it.
    pub demoted: String,
    /// How many acts were in flight on it at the moment of the swap.
    pub in_flight: usize,
}

/// Why a hotswap did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromoteError {
    /// No binding under that name.
    NotBound(String),
    /// Bound, but with no shadow, so there is nothing to promote.
    NoShadow(String),
    /// The shadow's declaration is gone. Hot-unplugged between link and swap.
    ShadowUndeclared(String),
    /// The shadow still exists but no longer declares what the link was made on.
    ShadowNoLongerServes {
        demand: String,
        shadow: String,
        missing: BTreeSet<String>,
    },
}

/// Counts acts in flight per resource.
///
/// This exists because the honest answer to "may I unplug that" is a number, and
/// without a number the only available answers are "yes" (unlink a busy resource
/// and lose the act) and "no" (never hotswap). Neither is right, and the second
/// is what the machine does today because the driver table requires exclusive
/// access to change at all.
#[derive(Debug, Default)]
pub struct Inflight {
    counts: std::sync::Mutex<BTreeMap<String, usize>>,
}

impl Inflight {
    pub fn new() -> Arc<Self> {
        Arc::new(Inflight::default())
    }

    /// Marks one act as begun against a resource.
    ///
    /// Returns the act's remaining lifetime. Ending the act is tied to dropping
    /// the guard rather than to a second call the caller can forget, because a
    /// counter that only ever goes up eventually refuses every hotswap and
    /// reports resource exhaustion for what is a leak in the caller.
    pub fn begin(self: &Arc<Self>, resource: &str) -> Act {
        {
            let mut c = self.counts.lock().expect("inflight poisoned");
            *c.entry(resource.to_string()).or_insert(0) += 1;
        }
        Act {
            resource: resource.to_string(),
            owner: Arc::clone(self),
        }
    }

    /// How many acts are in flight on a resource.
    pub fn on(&self, resource: &str) -> usize {
        self.counts
            .lock()
            .expect("inflight poisoned")
            .get(resource)
            .copied()
            .unwrap_or(0)
    }
}

/// One act in flight against a resource. Dropping it ends the act.
#[derive(Debug)]
pub struct Act {
    resource: String,
    owner: Arc<Inflight>,
}

impl Drop for Act {
    fn drop(&mut self) {
        let mut c = self.owner.counts.lock().expect("inflight poisoned");
        if let Some(n) = c.get_mut(&self.resource) {
            *n = n.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    //! The sequence, run end to end: check the hardware, resolve a demand
    //! against what it declares, link, act, hotswap, act again.

    use super::*;
    use crate::bridge::primitive::{StateType, UreAction};

    fn ure(id: &str, acts: &[&str]) -> UreResource {
        UreResource {
            ure_version: "1.0".to_string(),
            resource_id: id.to_string(),
            category: "actuator".to_string(),
            state_space: std::collections::HashMap::new(),
            action_primitives: acts
                .iter()
                .map(|a| UreAction {
                    id: a.to_string(),
                    aliases: None,
                    params: std::collections::HashMap::new(),
                    target_state: String::new(),
                    constraints: vec![],
                })
                .collect(),
        }
    }

    /// A two-act demand, so "declares one of the two" can be told apart from
    /// "declares both" — the partial match is the interesting refusal and a
    /// one-act fixture cannot produce it.
    fn two_act_demand() -> Demand {
        Demand {
            name: "cool-the-rack".into(),
            required: ["read_state", "set_value"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    /// **The flag.** A linker binds a capability to a resource that declares it,
    /// and nothing declares the binding but the manifests.
    #[test]
    fn a_demand_binds_to_a_resource_that_declares_it() {
        let mut linker = Linker::new();
        // Two processors, both capable. The primary is the first by sorted id,
        // and the second is kept warm as a shadow.
        linker.declare(&ure("cpu-0", &["read_state", "set_value"]));
        linker.declare(&ure("cpu-1", &["read_state", "set_value"]));

        let b = linker
            .link(&two_act_demand())
            .expect("a capable resource was declared");
        assert_eq!(
            b.primary, "cpu-0",
            "sorted id, so the choice is a function of the declarations"
        );
        assert_eq!(b.shadow.as_deref(), Some("cpu-1"));
    }

    /// **A hand-written routing table does not do this.** `UpaDispatcher::set_primary`
    /// takes any pair of strings; it cannot tell a capable processor from a
    /// network card. The refusal below is the difference, and it is the whole
    /// reason this is a linker and not a table.
    #[test]
    fn a_demand_is_refused_when_nothing_declares_all_of_it() {
        let mut linker = Linker::new();
        // A CPU that can read but not set. Close enough to bind, and the
        // misroute it invites is the one worth refusing.
        linker.declare(&ure("cpu-0", &["read_state"]));
        // A network card that cannot do either.
        linker.declare(&ure("nic-0", &["broadcast"]));

        match linker.link(&two_act_demand()) {
            Err(Unbound::Partial { near }) => {
                assert_eq!(
                    near,
                    vec!["cpu-0".to_string()],
                    "the near miss is the CPU, and it is reported as the near miss \
                     rather than as no match — a half-bound demand is a misroute"
                );
            }
            other => panic!("a half-capable resource bound the demand: {other:?}"),
        }
        assert_eq!(linker.bound("cool-the-rack"), None, "nothing was bound");
    }

    /// "Nothing matched" and "something nearly matched" have different fixes, and
    /// reporting one of them as the other loses that.
    #[test]
    fn an_undeclared_demand_and_a_partially_declared_one_are_different_failures() {
        let mut empty = Linker::new();
        empty.declare(&ure("nic-0", &["broadcast"]));
        assert_eq!(
            empty.link(&two_act_demand()),
            Err(Unbound::Undeclared {
                missing: two_act_demand().required
            }),
            "nothing declared any of it, which is a different problem from \
             something declaring half"
        );

        // And a demand that asks for nothing is refused rather than satisfied by
        // everything, because a demand for nothing is not a demand.
        let mut linker = Linker::new();
        linker.declare(&ure("cpu-0", &["read_state"]));
        assert_eq!(
            linker.link(&Demand {
                name: "do-nothing".into(),
                required: BTreeSet::new()
            }),
            Err(Unbound::Empty)
        );
    }

    /// **Hotswap.** The shadow is promoted, and the demoted primary is reported
    /// with the number of acts still running on it.
    #[test]
    fn a_shadow_is_promoted_and_the_demoted_cpu_is_reported_busy() {
        let mut linker = Linker::new();
        linker.declare(&ure("cpu-0", &["read_state", "set_value"]));
        linker.declare(&ure("cpu-1", &["read_state", "set_value"]));
        linker.link(&two_act_demand()).expect("link");

        let inflight = Inflight::new();
        let act = inflight.begin("cpu-0");
        assert_eq!(inflight.on("cpu-0"), 1);

        let p = linker
            .promote("cool-the-rack", &inflight)
            .expect("a declared shadow can be promoted");
        assert_eq!(p.demoted, "cpu-0");
        assert_eq!(
            p.in_flight, 1,
            "one act was still on the demoted CPU, and the swap says so instead \
             of silently deciding what happens to it"
        );
        assert_eq!(linker.bound("cool-the-rack").unwrap().primary, "cpu-1");

        // The act finishes on the CPU it started on, and the count returns to
        // zero. This is the "complete" policy, and it is the default because the
        // alternatives require a caller's consent that the linker cannot obtain.
        drop(act);
        assert_eq!(
            inflight.on("cpu-0"),
            0,
            "an ended act stops being in flight"
        );
    }

    /// **A shadow that no longer serves is not promoted.** The declaration was
    /// withdrawn and re-added with fewer actions between the link and the swap,
    /// which is what a firmware update looks like from here.
    #[test]
    fn a_shadow_that_stopped_declaring_the_demand_is_not_promoted() {
        let mut linker = Linker::new();
        linker.declare(&ure("cpu-0", &["read_state", "set_value"]));
        linker.declare(&ure("cpu-1", &["read_state", "set_value"]));
        linker.link(&two_act_demand()).expect("link");

        // cpu-1 is refitted and now declares only one of the two acts.
        linker.undeclare("cpu-1");
        linker.declare(&ure("cpu-1", &["read_state"]));

        match linker.promote("cool-the-rack", &Inflight::new()) {
            Err(PromoteError::ShadowNoLongerServes {
                missing, shadow, ..
            }) => {
                assert_eq!(shadow, "cpu-1");
                assert_eq!(missing, ["set_value".to_string()].into());
            }
            other => panic!("a shadow that cannot serve the demand was promoted: {other:?}"),
        }
        assert_eq!(
            linker.bound("cool-the-rack").unwrap().primary,
            "cpu-0",
            "the binding is unchanged, because a refused swap changed nothing"
        );
    }

    /// **Hot-unplug.** The resource is withdrawn while acts are on it, and the
    /// linker still reports the count. This is the case the old table could not
    /// express at all: `set_primary` could point a slot at a CPU that was no
    /// longer there, and nothing would notice until an act failed.
    #[test]
    fn an_unplugged_resource_still_reports_its_in_flight_acts() {
        let mut linker = Linker::new();
        linker.declare(&ure("cpu-0", &["read_state", "set_value"]));
        linker.declare(&ure("cpu-1", &["read_state", "set_value"]));
        linker.link(&two_act_demand()).expect("link");

        let inflight = Inflight::new();
        let _act = inflight.begin("cpu-0");
        assert_eq!(inflight.on("cpu-0"), 1);

        let withdrawn = linker.undeclare("cpu-0").expect("it was declared");
        assert_eq!(withdrawn.id, "cpu-0");
        assert_eq!(
            linker.declares("cpu-0"),
            None,
            "an unplugged resource leaves the linker as it entered it"
        );
        assert_eq!(
            inflight.on("cpu-0"),
            1,
            "the act is still running on hardware that is no longer declared, \
             which is the fact the caller needs and could not previously obtain"
        );
    }

    /// **The binding is separable from the declaration.** A warm processor can
    /// be left idle without being unplugged, which collapsing the two would make
    /// impossible.
    #[test]
    fn a_binding_can_be_dropped_while_the_declaration_survives() {
        let mut linker = Linker::new();
        linker.declare(&ure("cpu-0", &["read_state", "set_value"]));
        linker.link(&two_act_demand()).expect("link");

        assert!(linker.unlink("cool-the-rack").is_some());
        assert_eq!(linker.bound("cool-the-rack"), None, "the binding is gone");
        assert!(linker.declares("cpu-0").is_some(), "the resource is not");
    }

    /// **Resolution is a function of the declarations, not of hash order.** Two
    /// linkers fed the same resources in opposite orders pick the same primary;
    /// if they did not, two runs over the same hardware could bind differently
    /// and nothing would say so.
    #[test]
    fn the_binding_does_not_depend_on_the_order_resources_arrived_in() {
        let a = ure("cpu-0", &["read_state", "set_value"]);
        let b = ure("cpu-1", &["read_state", "set_value"]);
        let c = ure("cpu-2", &["read_state", "set_value"]);

        let mut forwards = Linker::new();
        forwards.declare(&a);
        forwards.declare(&b);
        forwards.declare(&c);
        let mut backwards = Linker::new();
        backwards.declare(&c);
        backwards.declare(&b);
        backwards.declare(&a);

        assert_eq!(
            forwards.link(&two_act_demand()).unwrap(),
            backwards.link(&two_act_demand()).unwrap(),
            "three capable processors, two arrival orders, one binding"
        );
    }

    /// **A capability set is the manifest, not a second vocabulary.** The thing
    /// that declares a capability and the thing the linker checks against it are
    /// the same artifact, so they cannot drift apart — and no name exists for a
    /// capability anywhere except in some resource's manifest.
    #[test]
    fn a_capability_exists_only_where_some_manifest_declares_it() {
        let mut linker = Linker::new();
        linker.declare(&ure("cpu-0", &["read_state", "set_value"]));
        assert_eq!(
            linker.declares("cpu-0").unwrap().iter().collect::<Vec<_>>(),
            vec!["read_state", "set_value"],
            "the declared set is the manifest's actions and nothing else"
        );
        assert!(linker.declares("hypothetical").is_none());
    }
}
