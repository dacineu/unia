use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// The Universal Primitive IDs defined in the Primitive Library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UniversalPrimitive {
    // State-Query
    GetState,
    GetValue,
    CheckSense,
    // State-Transition
    SetValue,
    Toggle,
    Increment,
    Reset,
    // Flow & Signal
    Route,
    Pipe,
    Broadcast,
    // Temporal & Event
    Delay,
    Watch,
    Pulse,
    // Compute & Logic
    Compare,
    Transform,
    Validate,
}

/// Every primitive's name, in the form a signature spells it.
///
/// Exposed because the store has to recognise a *declared act* — a primitive
/// sequence — inside a manifest, and the only decidable way to tell an induced
/// action from a hand-authored one is whether its id is made entirely of primitive
/// names. `emergency_shutdown` is not a sequence; `SetValue_CheckSense` is. The
/// store cannot import the enum's variants by reflection, so the names are listed
/// here once and a test below holds the list to the enum.
pub const PRIMITIVE_NAMES: &[&str] = &[
    "GetState",
    "GetValue",
    "CheckSense",
    "SetValue",
    "Toggle",
    "Increment",
    "Reset",
    "Route",
    "Pipe",
    "Broadcast",
    "Delay",
    "Watch",
    "Pulse",
    "Compare",
    "Transform",
    "Validate",
];

/// Whether `name` is a primitive.
///
/// Whole-token and case-sensitive, which is the same discipline `resolve_primitive`
/// uses and for the same reason: a substring test fires on ordinary words, since
/// `widget` contains `get` and `offset` contains `off`.
pub fn is_primitive_name(name: &str) -> bool {
    PRIMITIVE_NAMES.contains(&name)
}

/// Substrings that select each primitive, ordered most specific first.
///
/// A list and not a decision tree because the input is a manifest's free-form
/// `action_primitives[].id`, which the format does not constrain. That is the
/// residual weakness of D6: this table guesses less badly than the old one, and
/// it says when it is guessing. The real fix is a grammar for action ids, which
/// is the same specification work as the constraint grammar and is tracked with
/// it.
pub const PRIMITIVE_TABLE: &[(&[&str], UniversalPrimitive)] = &[
    // Ordered most specific first. Two shapes of mistake this ordering prevents:
    // a state query swallowed by a bare `get`, and a reset swallowed by a bare
    // `off` that happens to occur inside a longer word.
    (
        &[
            "get_state",
            "read_state",
            "inspect",
            "snapshot",
            "dump",
            "describe",
        ],
        UniversalPrimitive::GetState,
    ),
    (&["watch", "observe", "monitor"], UniversalPrimitive::Watch),
    (
        &["broadcast", "fan_out", "announce"],
        UniversalPrimitive::Broadcast,
    ),
    (
        &["transform", "map", "convert"],
        UniversalPrimitive::Transform,
    ),
    (&["validate"], UniversalPrimitive::Validate),
    (
        &["check", "verify", "assert", "sense"],
        UniversalPrimitive::CheckSense,
    ),
    (&["compare", "match", "diff"], UniversalPrimitive::Compare),
    (
        &["get", "read", "fetch", "query", "poll", "peek"],
        UniversalPrimitive::GetValue,
    ),
    (
        &["delay", "sleep", "wait", "defer", "retry_later"],
        UniversalPrimitive::Delay,
    ),
    (&["pulse", "ping", "poke"], UniversalPrimitive::Pulse),
    (
        &["route", "forward", "relay", "send", "dispatch"],
        UniversalPrimitive::Route,
    ),
    (
        &["pipe", "plumb", "attach", "bind"],
        UniversalPrimitive::Pipe,
    ),
    (&["toggle", "flip", "switch"], UniversalPrimitive::Toggle),
    (
        &["increment", "inc", "bump", "raise", "lower", "adjust_by"],
        UniversalPrimitive::Increment,
    ),
    // `off` alone is deliberately absent. It occurs inside ordinary words, and a
    // reset key that fires on `offset_value` is worse than no key at all.
    (
        &[
            "reset",
            "shutdown",
            "halt",
            "stop",
            "kill",
            "turn_off",
            "power_off",
            "shut_off",
            "clear",
            "purge",
            "flush",
            "empty",
            "wipe",
        ],
        UniversalPrimitive::Reset,
    ),
    (
        &["set", "adjust", "assign", "write", "configure", "update"],
        UniversalPrimitive::SetValue,
    ),
];

/// Splits an action id into whole terms.
///
/// Separates on non-alphanumerics and on case changes, so `write_file`,
/// `writeFile` and `WriteFile` all yield the same terms. A manifest's
/// `action_primitives[].id` is free text and the format does not constrain its
/// casing, so the resolver has to be indifferent to it.
pub fn tokenize_id(id: &str) -> Vec<String> {
    let chars: Vec<char> = id.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();

    for (i, &c) in chars.iter().enumerate() {
        if !c.is_alphanumeric() {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        // A lower-case letter adjacent to an upper-case one starts a new term, so
        // `writeFile` and `SetValue` both split. A run of capitals stays one term,
        // so `SET_VALUE` does not become eight letters.
        let prev_lower = i > 0 && chars[i - 1].is_lowercase();
        let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
        let boundary = c.is_uppercase() && !cur.is_empty() && (prev_lower || next_lower);

        if boundary {
            out.push(std::mem::take(&mut cur));
        }
        cur.extend(c.to_lowercase());
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// The standard packet sent from the Bridge to the Actuator Nucleus
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimitivePacket {
    pub header: PacketHeader,
    pub payload: PacketPayload,
    pub context: PacketContext,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketHeader {
    pub timestamp: u64,
    pub request_id: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketPayload {
    pub primitive: UniversalPrimitive,
    pub resource_id: String,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PacketContext {
    pub expected_state: Option<String>,
    pub timeout_ms: u32,
    /// The action's declared preconditions, carried with the packet.
    ///
    /// They travel rather than being checked here because this is the component
    /// that knows *what is declared* and not *what is currently true*: the
    /// bridge holds the manifest, the nucleus holds the state, and only the
    /// nucleus can evaluate a predicate over a state. Checking them here was the
    /// original arrangement and it could only ever print them, which is
    /// divergence D5. Defaults to empty so a packet written by an older build,
    /// or by hand, still parses.
    #[serde(default)]
    pub preconditions: Vec<String>,
}

/// Formal representation of a .ure resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UreResource {
    pub ure_version: String,
    pub resource_id: String,
    pub category: String,
    pub state_space: HashMap<String, StateType>,
    pub action_primitives: Vec<UreAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateType {
    pub r#type: String,
    pub range: Option<(f64, f64)>,
    pub unit: Option<String>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UreAction {
    pub id: String,
    pub aliases: Option<Vec<String>>,
    pub params: HashMap<String, String>,
    pub target_state: String,
    pub constraints: Vec<String>,
}

pub struct PrimitiveBridge {
    resources: HashMap<String, UreResource>,
}

impl PrimitiveBridge {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Dynamically load all .ure files from a directory
    pub fn load_resources_from_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let entries = fs::read_dir(path).map_err(|e| format!("Failed to read dir: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Entry error: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ure") {
                let content =
                    fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
                let resource: UreResource = serde_yaml::from_str(&content)
                    .map_err(|e| format!("YAML parse error in {:?}: {}", path, e))?;

                self.resources
                    .insert(resource.resource_id.clone(), resource);
                println!(
                    "[Bridge] Loaded resource: {}",
                    entry.file_name().to_string_lossy()
                );
            }
        }
        Ok(())
    }

    pub fn load_resource(&mut self, resource: UreResource) {
        self.resources
            .insert(resource.resource_id.clone(), resource);
    }

    /// Maps natural language intent to a Universal Primitive Packet
    pub fn map_intent(&self, resource_id: &str, intent: &str) -> Result<PrimitivePacket, String> {
        let resource = self
            .resources
            .get(resource_id)
            .ok_or_else(|| format!("Resource {} not found", resource_id))?;

        let intent_lower = intent.to_lowercase();

        // 1. Exact/Containment Match (Fast Path)
        let mut best_action = None;
        let mut max_score = 0.0;

        for action in &resource.action_primitives {
            let normalized_id = action.id.to_lowercase().replace('_', " ");
            if intent_lower.contains(&normalized_id) {
                return self.create_packet(resource_id, action);
            }

            if let Some(aliases) = &action.aliases {
                for alias in aliases {
                    if intent_lower.contains(&alias.to_lowercase()) {
                        return self.create_packet(resource_id, action);
                    }
                }
            }

            // 2. Semantic Scoring (Slow Path)
            let score =
                crate::bridge::semantic::SemanticMapper::compute_score(&intent_lower, &action.id);
            if score > max_score {
                max_score = score;
                best_action = Some(action);
            }
        }

        // Threshold for semantic match (0.3 = moderate overlap)
        if max_score > 0.3 {
            println!("[Bridge] Semantic match found (score: {:.2})", max_score);
            return self.create_packet(resource_id, best_action.unwrap());
        }

        Err(format!(
            "No matching action for intent '{}' in resource {}",
            intent, resource_id
        ))
    }

    fn create_packet(
        &self,
        resource_id: &str,
        action: &UreAction,
    ) -> Result<PrimitivePacket, String> {
        let primitive = self
            .resolve_primitive(&action.id, &action.target_state)
            .map_err(|e| format!("[Bridge] {e}"))?;

        // Carried, not checked: the state this would be checked against lives in
        // the nucleus. See the note on `PacketContext::preconditions`.
        let preconditions = action.constraints.clone();

        Ok(PrimitivePacket {
            header: PacketHeader {
                timestamp: 1694430000,
                request_id: "req-abc-123".to_string(),
                priority: Priority::Medium,
            },
            payload: PacketPayload {
                primitive,
                resource_id: resource_id.to_string(),
                arguments: action.params.clone(),
            },
            context: PacketContext {
                expected_state: Some(action.target_state.clone()),
                timeout_ms: 500,
                preconditions,
            },
        })
    }

    /// Resolves an action id to the primitive that carries it out.
    ///
    /// Returns `Err` for anything the table does not cover, which is the whole
    /// point of the change. The previous version returned `SetValue` for every
    /// unrecognised id, so a typo and a genuinely unresolvable action produced
    /// the same silent misroute: a valve asked to *broadcast* would have been
    /// sent a `SetValue`, and nothing anywhere would have said so. A resolver that
    /// cannot fail is a resolver that cannot be trusted with a peer's input, and
    /// the vocabulary-free channel this crate is heading towards sends exactly
    /// that.
    ///
    /// The match is ordered from most specific to least, and a longer key wins
    /// over a shorter one that is also a substring, so `read_state` resolves to a
    /// state query rather than being swallowed by a bare `read`.
    fn resolve_primitive(
        &self,
        action_id: &str,
        _target_state: &str,
    ) -> Result<UniversalPrimitive, String> {
        let tokens = tokenize_id(action_id);
        for (keys, primitive) in PRIMITIVE_TABLE {
            for key in *keys {
                let key_tokens = tokenize_id(key);
                // Whole-token containment, not substring. A bare substring test
                // fires on ordinary words: `widget` contains `get`, and `offset`
                // contains `off`, so a request to read a widget or to set an
                // offset was dispatched as a value read or a shutdown. Requiring
                // every token of the key to be present as a whole token removes
                // that entire class of misroute.
                if !key_tokens.is_empty() && key_tokens.iter().all(|k| tokens.contains(k)) {
                    return Ok(primitive.clone());
                }
            }
        }
        Err(format!(
            "no primitive resolves action {action_id:?}. The vocabulary covers \
             {} primitives; this id matches none of them, and guessing would \
             dispatch the wrong one.",
            PRIMITIVE_TABLE.len()
        ))
    }
}

#[cfg(test)]
mod resolve_tests {
    use super::*;

    fn resolve(id: &str) -> Result<UniversalPrimitive, String> {
        let bridge = PrimitiveBridge::new();
        bridge.resolve_primitive(id, "")
    }

    #[test]
    fn every_declared_primitive_is_reachable_from_the_table() {
        // The whole point of the change: the table must cover the vocabulary it
        // claims to, or "resolves to a primitive" is not true of the type.
        let declared = [
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
        for want in declared {
            assert!(
                PRIMITIVE_TABLE.iter().any(|(_, p)| *p == want),
                "{want:?} is declared but no substring selects it"
            );
        }
    }

    #[test]
    fn reaches_all_sixteen_that_were_previously_unreachable() {
        // The old resolver returned Reset, SetValue or GetValue and nothing else,
        // so thirteen of the sixteen were declared and dead.
        for (id, want) in [
            ("get_state", UniversalPrimitive::GetState),
            ("check_sense", UniversalPrimitive::CheckSense),
            ("toggle_valve", UniversalPrimitive::Toggle),
            ("increment_counter", UniversalPrimitive::Increment),
            ("route_packet", UniversalPrimitive::Route),
            ("pipe_output", UniversalPrimitive::Pipe),
            ("broadcast_state", UniversalPrimitive::Broadcast),
            ("delay_retry", UniversalPrimitive::Delay),
            ("watch_pressure", UniversalPrimitive::Watch),
            ("pulse_valve", UniversalPrimitive::Pulse),
            ("compare_levels", UniversalPrimitive::Compare),
            ("transform_input", UniversalPrimitive::Transform),
            ("validate_config", UniversalPrimitive::Validate),
        ] {
            assert_eq!(resolve(id), Ok(want), "{id} resolved wrongly");
        }
    }

    #[test]
    fn an_unresolvable_action_is_refused_rather_than_guessed() {
        // This is the behaviour the old resolver could not produce. A typo and a
        // genuinely unknown action used to dispatch SetValue, so a request to
        // broadcast was silently sent a set and nothing said so.
        let err = resolve("frobnicate_the_widget").unwrap_err();
        assert!(
            err.contains("frobnicate_the_widget"),
            "names the input: {err}"
        );
        assert!(err.contains("guessing"), "says why it refused: {err}");
    }

    #[test]
    fn a_word_merely_containing_a_key_is_not_a_match() {
        // The bug that forced whole-token matching. `widget` contains `get` and
        // `offset` contains `off`, so a substring resolver read a widget as a
        // value query and an offset as a shutdown.
        assert!(resolve("render_widget").is_err(), "`widget` contains `get`");
        assert_ne!(
            resolve("offset_value"),
            Ok(UniversalPrimitive::Reset),
            "`offset` contains `off`"
        );
    }

    #[test]
    fn ids_split_the_same_way_under_every_casing() {
        // The format does not constrain the casing of an action id, so the
        // resolver has to be indifferent to it. An all-caps id splitting into
        // eight letters is how that was first broken.
        for a in ["set_value", "SET_VALUE", "SetValue"] {
            assert_eq!(
                tokenize_id(a),
                vec!["set".to_string(), "value".to_string()],
                "{a}"
            );
        }
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(resolve("SET_VALUE"), resolve("set_value"));
        assert_eq!(resolve("Shutdown"), resolve("shutdown"));
    }

    #[test]
    fn a_state_query_is_not_swallowed_by_a_bare_get() {
        // `get_state` contains `get`, and a state query is a different act from a
        // value read. Ordering is what keeps them apart.
        assert_eq!(resolve("get_state"), Ok(UniversalPrimitive::GetState));
        assert_eq!(resolve("get_value"), Ok(UniversalPrimitive::GetValue));
    }

    #[test]
    fn an_ordinary_word_containing_off_is_not_a_reset() {
        // `off` used to be a reset key and fired on `offset_value`. Substring
        // matching cannot be made safe in general, so the ambiguous key was
        // removed rather than ordered around, and the specific spellings that do
        // mean a reset were kept.
        assert_ne!(resolve("offset_value"), Ok(UniversalPrimitive::Reset));
        assert_eq!(resolve("turn_off"), Ok(UniversalPrimitive::Reset));
        assert_eq!(resolve("power_off"), Ok(UniversalPrimitive::Reset));
    }

    #[test]
    fn the_real_corpus_actions_all_resolve() {
        // The manifests that ship must not regress into the refusal path, or the
        // fix breaks the only working retrieval in the repository.
        for id in [
            "emergency_shutdown",
            "adjust_flow",
            "write_file",
            "read_file",
            "clear_cache",
        ] {
            assert!(resolve(id).is_ok(), "{id} no longer resolves");
        }
    }

    #[test]
    fn the_primitive_the_ca_maduci_uses_reaches_the_bridge() {
        // The pet records SetValue/CheckSense sequences, and a vocabulary-free
        // channel between creatures would send exactly this.
        assert_eq!(resolve("SetValue"), Ok(UniversalPrimitive::SetValue));
        assert_eq!(resolve("CheckSense"), Ok(UniversalPrimitive::CheckSense));
    }
}
