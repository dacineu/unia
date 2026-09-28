use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    /// Why this primitive is not funded by a command, or `None` when it is.
    ///
    /// **The commons is a set of promises, and a promise is only as good as its
    /// binding.** This is where the project's real state is recorded: a primitive
    /// with no `os_mapping` has a bond that cannot be called, and the honest
    /// response to that is to say which and why rather than to leave a gap that
    /// reads as an oversight.
    pub unfunded_reason: Option<String>,
}

pub struct PrimitiveBridge {
    nuclei: HashMap<String, PrimitiveNucleus>,
    current_os: OsVariant,
}

/// What a primitive is *for*, in one line, and whether a command can do it.
///
/// Recorded next to the binding because the reason six of the sixteen are unfunded
/// is not that they were forgotten. It is that they are not operations on a
/// machine: `Toggle` is inversion, `Increment` is arithmetic, `Pipe` is shell
/// syntax, `Broadcast` is a network operation and `Pulse` is a scheduled signal.
/// A command line cannot express any of them, and pretending otherwise with a
/// plausible-looking string would issue a bond that defaults on first use.
const CATALOGUE: &[(&str, &[(&OsVariant, &str)], &str)] = &[
    // A `GenericLinux` entry is a promise about the Linux family and nothing else.
    // It used to be a fallback for every platform, which meant `cat` resolved on
    // Windows -- a bond quietly holding on the wrong machine, which is the whole
    // failure this catalogue exists to record rather than hide.
    //
    // A promise is per-machine. Where macOS or Windows has the same tool under the
    // same name it is listed; where it has a different one, that is listed; where
    // it has none, the primitive is unfunded on that platform and says so.
    // --- State-Query -----------------------------------------------------
    (
        "GetState",
        &[
            (&OsVariant::GenericLinux, "cat"),
            (&OsVariant::MacOS, "cat"),
            (&OsVariant::Windows, "type"),
        ],
        "",
    ),
    (
        "GetValue",
        &[
            (&OsVariant::GenericLinux, "grep"),
            (&OsVariant::MacOS, "grep"),
            (&OsVariant::Windows, "findstr"),
        ],
        "",
    ),
    (
        "CheckSense",
        &[
            (&OsVariant::GenericLinux, "test"),
            (&OsVariant::MacOS, "test"),
            (&OsVariant::Windows, "findstr"),
        ],
        "",
    ),
    // --- State-Transition ------------------------------------------------
    (
        "SetValue",
        &[
            (&OsVariant::GenericLinux, "tee"),
            (&OsVariant::MacOS, "tee"),
        ],
        "no portable write on cmd.exe; Windows binds this through a driver",
    ),
    (
        "Toggle",
        &[],
        "inversion is not a command; it is a driver operation",
    ),
    ("Increment", &[], "arithmetic on a value is not a command"),
    (
        "Reset",
        &[
            (&OsVariant::GenericLinux, "truncate -s 0"),
            (&OsVariant::MacOS, "truncate -s 0"),
        ],
        "no portable truncate on cmd.exe; Windows binds this through a driver",
    ),
    // --- Flow & Signal ---------------------------------------------------
    (
        "Route",
        &[
            (&OsVariant::GenericLinux, "ip route"),
            (&OsVariant::Windows, "route print"),
        ],
        "macOS has no `ip route`; it binds through a driver",
    ),
    ("Pipe", &[], "a pipe is shell syntax, not an executable"),
    ("Broadcast", &[], "a network operation, not a command"),
    // --- Temporal & Event ------------------------------------------------
    (
        "Delay",
        &[
            (&OsVariant::GenericLinux, "sleep"),
            (&OsVariant::MacOS, "sleep"),
            (&OsVariant::Windows, "timeout"),
        ],
        "",
    ),
    (
        "Watch",
        &[(&OsVariant::GenericLinux, "inotifywait")],
        "Linux-only, and needs inotify-tools installed; macOS and Windows bind \
         this through a driver",
    ),
    ("Pulse", &[], "a scheduled signal, not a command"),
    // --- Compute & Logic -------------------------------------------------
    (
        "Compare",
        &[
            (&OsVariant::GenericLinux, "diff"),
            (&OsVariant::MacOS, "diff"),
            (&OsVariant::Windows, "fc"),
        ],
        "",
    ),
    (
        "Transform",
        &[
            (&OsVariant::GenericLinux, "sed"),
            (&OsVariant::MacOS, "sed"),
        ],
        "no portable stream editor on cmd.exe; Windows binds this through a driver",
    ),
    (
        "Validate",
        &[],
        "would need jq or equivalent, which is a dependency rather than a standard \
         tool, so it is unfunded on every platform until a driver provides it",
    ),
];

/// Whether a variant is in the Linux family, and so may fall back to a
/// `GenericLinux` binding.
///
/// An explicit list rather than "anything that is not Windows or MacOS", because a
/// new variant should default to *not* inheriting a Linux promise until someone
/// says it does. The failure here is a bond held on the wrong machine, and a
/// permissive default is how that happens.
fn is_linux_family(os: OsVariant) -> bool {
    matches!(
        os,
        OsVariant::Debian | OsVariant::Arch | OsVariant::Fedora | OsVariant::GenericLinux
    )
}

impl PrimitiveBridge {
    pub fn new(os: OsVariant) -> Self {
        let mut nuclei = HashMap::new();

        for (universal_id, bindings, unfunded) in CATALOGUE {
            let mut os_mapping = HashMap::new();
            for (variant, command) in bindings.iter() {
                // A binding named for a platform applies to that platform, and a
                // bare `GenericLinux` one is recorded as a *name to be spelled out
                // per platform later* rather than silently inherited — a single
                // command is a promise about one machine, and pretending it holds
                // everywhere is the failure the whole catalogue is about.
                os_mapping.insert(**variant, command.to_string());
            }
            nuclei.insert(
                universal_id.to_string(),
                PrimitiveNucleus {
                    universal_id: universal_id.to_string(),
                    os_mapping,
                    is_replaced_by_actuator: false,
                    replacement_uuid: None,
                    unfunded_reason: if unfunded.is_empty() {
                        None
                    } else {
                        Some(unfunded.to_string())
                    },
                },
            );
        }

        Self {
            nuclei,
            current_os: os,
        }
    }

    /// Every primitive this bridge knows, in declaration order.
    pub fn universal_ids(&self) -> Vec<&str> {
        CATALOGUE.iter().map(|(id, _, _)| *id).collect()
    }

    /// How many primitives a command can actually call on this platform.
    ///
    /// The number the paper should quote if it quotes one. It is deliberately a
    /// count rather than a boolean, because a commons that reports itself as
    /// "available" while six of its sixteen promises default on first use is not
    /// describing itself.
    pub fn funded_count(&self) -> usize {
        self.nuclei
            .values()
            .filter(|n| self.binding_for(n).is_some())
            .count()
    }

    /// The primitives with no command on this platform, and why.
    pub fn unfunded(&self) -> Vec<(&str, &str)> {
        CATALOGUE
            .iter()
            .filter_map(|(id, _, reason)| {
                if self.binding_for(self.nuclei.get(*id)?).is_none() {
                    Some((*id, *reason))
                } else {
                    None
                }
            })
            .collect()
    }

    /// The command this platform can call, if there is one.
    ///
    /// Owned rather than borrowed: the maps are small, the alternative is a
    /// lifetime on every call site, and a `&String` return from a `&self` method
    /// that also takes a `&PrimitiveNucleus` is a signature that says nothing about
    /// which of the two the result borrows from.
    fn binding_for(&self, n: &PrimitiveNucleus) -> Option<String> {
        n.os_mapping.get(&self.current_os).cloned().or_else(|| {
            // Only the Linux family inherits a `GenericLinux` binding. Every other
            // platform has to have been promised a command by name.
            is_linux_family(self.current_os)
                .then(|| n.os_mapping.get(&OsVariant::GenericLinux).cloned())
                .flatten()
        })
    }

    /// Resolves a primitive to its OS-specific command or to its unia Actuator.
    pub fn resolve(&self, universal_id: &str) -> Result<String, String> {
        let nucleus = self
            .nuclei
            .get(universal_id)
            .ok_or_else(|| format!("Primitive {universal_id} not found"))?;

        if nucleus.is_replaced_by_actuator {
            let uuid = nucleus
                .replacement_uuid
                .as_ref()
                .ok_or_else(|| format!("Primitive {universal_id} has no replacement"))?;
            return Ok(format!("actuate:{uuid}"));
        }

        // **Default, loudly.** The old version returned a bare "no mapping", which
        // reads as a lookup miss. It is not: the primitive is known and the bond
        // is unfunded, and a caller that has to distinguish "I have never heard of
        // this" from "I know this and cannot call it" is being told the wrong
        // thing by the old version.
        self.binding_for(nucleus).ok_or_else(|| {
            format!(
                "Primitive {universal_id} is known but unfunded on {:?}: {}",
                self.current_os,
                nucleus
                    .unfunded_reason
                    .as_deref()
                    .unwrap_or("no command for this platform")
            )
        })
    }

    /// Allows the Mutation Engine to replace an OS primitive with a specialized .ure actuator.
    pub fn replace_with_actuator(
        &mut self,
        universal_id: &str,
        actuator_uuid: Uuid,
    ) -> Result<(), String> {
        if let Some(nucleus) = self.nuclei.get_mut(universal_id) {
            nucleus.is_replaced_by_actuator = true;
            nucleus.replacement_uuid = Some(actuator_uuid);
            Ok(())
        } else {
            Err("Primitive not found".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::primitive::UniversalPrimitive;

    /// The catalogue is the whole of the commons, so a primitive the enum declares
    /// and the bridge does not is a promise nobody has issued.
    ///
    /// The file had **no tests at all** and seeded one primitive, so fifteen of the
    /// sixteen promises in the enum could not be called and nothing said so. This
    /// holds the two lists together.
    #[test]
    fn the_catalogue_covers_every_primitive_the_enum_declares() {
        let bridge = PrimitiveBridge::new(OsVariant::GenericLinux);
        let every = [
            UniversalPrimitive::GetState,
            UniversalPrimitive::GetValue,
            UniversalPrimitive::CheckSense,
            UniversalPrimitive::SetValue,
            UniversalPrimitive::Toggle,
            UniversalPrimitive::Increment,
            UniversalPrimitive::Reset,
            UniversalPrimitive::Route,
            UniversalPrimitive::Pipe,
            UniversalPrimitive::Broadcast,
            UniversalPrimitive::Delay,
            UniversalPrimitive::Watch,
            UniversalPrimitive::Pulse,
            UniversalPrimitive::Compare,
            UniversalPrimitive::Transform,
            UniversalPrimitive::Validate,
        ];
        let known: Vec<&str> = bridge.universal_ids();

        assert_eq!(
            known.len(),
            every.len(),
            "the catalogue holds {} and the enum declares {}",
            known.len(),
            every.len()
        );
        for p in every {
            let name = format!("{p:?}");
            assert!(
                known.contains(&name.as_str()),
                "{name} is declared and has no entry in the catalogue, so its bond \\
                 cannot be called and nothing would say so"
            );
        }
    }

    /// **The measurement.** How much of the commons is actually funded.
    ///
    /// Six of the sixteen are not, and the reason is recorded for each: they are
    /// not operations on a machine. `Toggle` is inversion, `Increment` is
    /// arithmetic, `Pipe` is shell syntax, `Broadcast` is networking, `Pulse` is a
    /// scheduled signal, and `Validate` would need a dependency rather than a
    /// standard tool. Binding them to plausible-looking strings would issue bonds
    /// that default on first use, which is worse than an honest gap.
    ///
    /// Asserted as the exact list rather than a count, so funding one of them is a
    /// deliberate act with a visible diff.
    #[test]
    fn six_of_the_sixteen_promises_are_unfunded_and_they_say_why() {
        let bridge = PrimitiveBridge::new(OsVariant::GenericLinux);
        let unfunded: Vec<&str> = bridge.unfunded().into_iter().map(|(id, _)| id).collect();

        assert_eq!(
            unfunded,
            vec![
                "Toggle",
                "Increment",
                "Pipe",
                "Broadcast",
                "Pulse",
                "Validate",
            ],
            "the set of unfunded promises changed: {unfunded:?}"
        );
        assert_eq!(
            bridge.funded_count(),
            10,
            "and the funded count no longer matches the unfunded list"
        );
        for (id, reason) in bridge.unfunded() {
            assert!(
                !reason.is_empty(),
                "{id} is unfunded and gives no reason, so the gap reads as an \\
                 oversight instead of a decision"
            );
        }
    }

    /// An unfunded primitive defaults *loudly*, and says it is known.
    ///
    /// The distinction that matters to a caller: never heard of this, versus knows
    /// this and cannot call it. The old error was `"No mapping for OS"`, which
    /// reads as the first.
    #[test]
    fn an_unfunded_primitive_is_known_and_says_it_cannot_be_called() {
        let bridge = PrimitiveBridge::new(OsVariant::GenericLinux);

        let err = bridge.resolve("Toggle").expect_err("Toggle has no command");
        assert!(
            err.contains("unfunded") && err.contains("inversion"),
            "the error does not distinguish an unfunded bond from an unknown one: {err}"
        );
        let unknown = bridge
            .resolve("Nonsense")
            .expect_err("not in the catalogue");
        assert!(
            unknown.contains("not found") && !unknown.contains("unfunded"),
            "an unknown primitive reported as unfunded, which is the opposite \\
             mistake and just as misleading: {unknown}"
        );
    }

    /// A promise is per-machine: the same primitive is a different command on a
    /// different platform, and a primitive with no command there defaults.
    ///
    /// This test first asserted that `GetState` was *unfunded* on Windows, on the
    /// reasoning that `cat` is not on cmd.exe. It is not — the catalogue promises
    /// `type` there, and the assertion was checking that a Linux binding leaked
    /// onto the wrong platform rather than that the platform had no promise. Which
    /// is the same failure as before, one level up: confusing "this platform has a
    /// different command" with "this platform has no command".
    #[test]
    fn a_promise_is_per_machine_and_defaults_where_there_is_none() {
        let linux = PrimitiveBridge::new(OsVariant::GenericLinux);
        let mac = PrimitiveBridge::new(OsVariant::MacOS);
        let windows = PrimitiveBridge::new(OsVariant::Windows);

        // The same primitive, three promises, none of them borrowed.
        assert_eq!(linux.resolve("GetState").as_deref(), Ok("cat"));
        assert_eq!(mac.resolve("GetState").as_deref(), Ok("cat"));
        assert_eq!(windows.resolve("GetState").as_deref(), Ok("type"));

        // And one that is promised on Linux and MacOS but not on Windows.
        assert_eq!(linux.resolve("SetValue").as_deref(), Ok("tee"));
        assert_eq!(mac.resolve("SetValue").as_deref(), Ok("tee"));
        let err = windows
            .resolve("SetValue")
            .expect_err("there is no portable write on cmd.exe");
        assert!(
            err.contains("unfunded") && err.contains("driver"),
            "the refusal does not say it is unfunded, nor that a driver is the \
             answer: {err}"
        );
    }

    /// The Linux family inherits; nothing else does.
    ///
    /// Debian, Arch and Fedora are not listed anywhere in the catalogue and must
    /// still resolve, or the bridge would work on a generic Linux box and fail on
    /// every real one.
    #[test]
    fn the_linux_family_inherits_the_generic_binding() {
        for os in [
            OsVariant::Debian,
            OsVariant::Arch,
            OsVariant::Fedora,
            OsVariant::GenericLinux,
        ] {
            let bridge = PrimitiveBridge::new(os);
            assert_eq!(
                bridge.resolve("GetState").as_deref(),
                Ok("cat"),
                "{os:?} did not inherit the generic binding"
            );
        }
    }

    /// An actuator can still fund an unfunded one, which is how the commons grows:
    /// not by renaming a primitive but by giving it a body.
    #[test]
    fn an_unfunded_primitive_can_be_funded_by_an_actuator() {
        let mut bridge = PrimitiveBridge::new(OsVariant::GenericLinux);
        assert!(bridge.resolve("Toggle").is_err());

        let uuid = Uuid::nil();
        bridge
            .replace_with_actuator("Toggle", uuid)
            .expect("Toggle is in the catalogue");

        assert_eq!(
            bridge.resolve("Toggle").as_deref(),
            Ok(format!("actuate:{uuid}").as_str()),
            "an actuator did not fund a primitive it was supposed to"
        );
        // And it stays the same primitive: the vocabulary is stable while the
        // binding is revisable, which is the whole design of the bridge.
        assert_eq!(bridge.universal_ids().len(), 16);
    }

    /// Replacing something the catalogue has never heard of is a refusal, not a
    /// silent success.
    #[test]
    fn an_actuator_cannot_fund_a_primitive_that_does_not_exist() {
        let mut bridge = PrimitiveBridge::new(OsVariant::GenericLinux);
        assert!(bridge
            .replace_with_actuator("Nonsense", Uuid::nil())
            .is_err());
    }
}
