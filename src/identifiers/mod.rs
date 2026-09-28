use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum UreIdError {
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("General error: {0}")]
    General(String),
}

/// The language-independent interior of a manifest.
///
/// This is what an artifact *is*, separated from how it is phrased. Identity is
/// computed over this and not over the whole body, so that the same capability
/// carries the same address whether it was expressed in English, Romanian, or a
/// mix, and so that learning a new phrasing does not change what a thing is.
///
/// Everything removed here is surface: the aliases a reader recognises it by, the
/// guidance written for a human, and any free-text description. Everything kept is
/// structure: which action primitives exist, what state they act on, and what
/// payload makes them runnable.
///
/// The alternative — hashing the whole body — makes identity depend on wording,
/// and has two consequences that are both fatal. A capability learned in two
/// languages becomes two artifacts, so it can never converge. And a rule becomes
/// a different rule the first time a player uses a phrase nobody had used before,
/// which is not a rare event but the normal one.
pub fn skeleton(manifest: &Value) -> Value {
    let mut out = serde_json::Map::new();

    // Keys that are surface rather than structure. A deny-list and not an
    // allow-list, deliberately: an allow-list silently drops any field it has not
    // heard of, so a manifest carrying something new would get an address that
    // ignores it and two genuinely different artifacts could collide. Unknown
    // keys stay in the skeleton until they are known to be prose.
    // `external_id` was missing from this list and the omission was live. A human
    // label was therefore hashed into the content address, so two byte-identical
    // manifests carrying different labels were different artifacts — the exact
    // defect the surface/structure split exists to prevent, reintroduced under a
    // second name. It was found by giving a digital creature a content address and
    // asking whether feeding it moved the answer: it did, because the address
    // included the name it was called.
    const SURFACE_KEYS: &[&str] = &[
        "resource_id",
        "external_id",
        "guidance",
        "description",
        "aliases",
    ];

    if let Some(obj) = manifest.as_object() {
        for (key, value) in obj {
            if SURFACE_KEYS.contains(&key.as_str()) {
                continue;
            }
            out.insert(key.clone(), value.clone());
        }

        // Action primitives reduced to their ids. Aliases are the surface and are
        // dropped; the id is the action.
        if let Some(actions) = obj.get("action_primitives").and_then(|a| a.as_array()) {
            let ids: Vec<Value> = actions
                .iter()
                .filter_map(|a| a.get("id").cloned())
                .collect();
            if !ids.is_empty() {
                out.insert("action_primitives".to_string(), Value::Array(ids));
            }
        }
    }

    Value::Object(out)
}

/// Deterministic URE-UUID (DU-UUID) implementation.
///
/// Generates a UUID from the *skeleton* of a .ure manifest, so identity is
/// independent of the language the manifest is written in.
pub struct DuUuid;

impl DuUuid {
    /// Generates a UUID from a .ure manifest.
    ///
    /// # Arguments
    /// * `manifest` - The .ure manifest as a JSON Value.
    /// * `encryption_key` - Optional 32-byte key for AES-256-GCM encryption.
    pub fn generate(
        manifest: &Value,
        encryption_key: Option<&[u8; 32]>,
    ) -> Result<Uuid, UreIdError> {
        // 1. Reduce to the language-independent interior.
        //
        // This also externalises the ID, because `resource_id` is not part of the
        // skeleton, so a resource cannot contain its own identity in its own
        // identity computation.
        let content = skeleton(manifest);

        // 2. Deterministic serialization.
        //
        // `serde_json::Map` is `BTreeMap`-backed and therefore already sorted
        // unless the `preserve_order` feature is enabled, in which case this
        // becomes non-deterministic and every address changes silently. The
        // comment that previously said the opposite was wrong; the risk it
        // described is real and is now asserted by a test rather than by prose.
        let serialized = serde_json::to_vec(&content)?;

        // 3. Payload Processing (Compression simulation & Encryption)
        // In a full implementation, a deterministic compression like zstd would go here.
        let payload = if let Some(key_bytes) = encryption_key {
            Self::encrypt_payload(&serialized, key_bytes)?
        } else {
            serialized
        };

        // 4. Hashing (SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let result = hasher.finalize();

        // 5. Map to UUID (First 16 bytes of SHA-256)
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[..16]);

        // Set Version 4 (0100) at index 6 (bits 48-51)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // Set Variant RFC 4122 (10xx) at index 8 (bits 64-65)
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Ok(Uuid::from_bytes(bytes))
    }

    fn encrypt_payload(data: &[u8], key_bytes: &[u8; 32]) -> Result<Vec<u8>, UreIdError> {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        // For deterministic UUIDs, the nonce must be deterministic.
        // WARNING: In standard encryption, nonces must NEVER be reused.
        // Here, since the purpose is generating a unique ID from content (not secrecy),
        // a fixed nonce is used to ensure the same content -> same UUID.
        let nonce = Nonce::from_slice(b"ure_det_nonc"); // exactly 12 bytes

        cipher
            .encrypt(nonce, data)
            .map_err(|e| UreIdError::Encryption(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deterministic_generation() {
        let manifest = json!({
            "resource_id": "old-id",
            "quality_of_product": 0.95,
            "checksum": "abc123hash"
        });

        let id1 = DuUuid::generate(&manifest, None).unwrap();
        let id2 = DuUuid::generate(&manifest, None).unwrap();

        assert_eq!(id1, id2);
        assert_eq!(id1.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_content_change_changes_id() {
        let manifest1 = json!({ "quality_of_product": 0.95 });
        let manifest2 = json!({ "quality_of_product": 0.96 });

        let id1 = DuUuid::generate(&manifest1, None).unwrap();
        let id2 = DuUuid::generate(&manifest2, None).unwrap();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_encryption_changes_id() {
        let manifest = json!({ "quality_of_product": 0.95 });
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];

        let id1 = DuUuid::generate(&manifest, Some(&key1)).unwrap();
        let id2 = DuUuid::generate(&manifest, Some(&key2)).unwrap();

        assert_ne!(id1, id2);
    }
}

#[cfg(test)]
mod skeleton_tests {
    use super::*;
    use serde_json::json;

    fn valve(aliases: [&str; 2]) -> Value {
        json!({
            "resource_id": "valve-001",
            "category": "actuator",
            "guidance": "A motorised valve.",
            "state_space": {"flow_rate": {"type": "float", "range": [0.0, 1.0]}},
            "action_primitives": [
                {"id": "emergency_shutdown", "aliases": aliases, "params": {},
                 "target_state": "flow_rate = 0.0", "constraints": []}
            ]
        })
    }

    #[test]
    fn drops_the_guidance_written_for_a_human() {
        let s = skeleton(&valve(["halt", "stop"]));
        assert!(
            s.get("guidance").is_none(),
            "guidance is prose, not structure"
        );
    }

    #[test]
    fn drops_aliases_but_keeps_the_action_they_name() {
        let s = skeleton(&valve(["halt", "stop"]));
        let actions = s["action_primitives"].as_array().unwrap();
        assert_eq!(actions[0], json!("emergency_shutdown"));
    }

    #[test]
    fn keeps_an_unrecognised_field_rather_than_dropping_it() {
        // A deny-list, not an allow-list. An allow-list silently discards fields
        // it has not heard of, so a manifest carrying something new would get an
        // address that ignores it and two different artifacts could collide.
        let m = json!({"resource_id": "x", "some_future_field": {"deeply": ["nested"]}});
        let s = skeleton(&m);
        assert_eq!(s["some_future_field"], json!({"deeply": ["nested"]}));
    }

    #[test]
    fn address_is_stable_across_key_order_in_the_manifest() {
        // serde_json::Map is BTreeMap-backed and therefore already sorted, so
        // this holds today. It stops holding the moment anyone enables the
        // `preserve_order` feature, and the failure would be silent: every
        // address in existence changes. Asserted here rather than left as a
        // comment, because a comment was what let the wrong claim stand.
        let a =
            json!({"category": "actuator", "payload": {"kind": "native"}, "ure_version": "1.0"});
        let b =
            json!({"ure_version": "1.0", "payload": {"kind": "native"}, "category": "actuator"});
        assert_eq!(
            DuUuid::generate(&a, None).unwrap(),
            DuUuid::generate(&b, None).unwrap()
        );
    }

    #[test]
    fn address_is_stable_across_key_order_inside_an_action() {
        let a = json!({"id": "halt", "target_state": "x", "constraints": []});
        let b = json!({"constraints": [], "target_state": "x", "id": "halt"});
        assert_eq!(
            DuUuid::generate(&a, None).unwrap(),
            DuUuid::generate(&b, None).unwrap()
        );
    }

    #[test]
    fn a_payload_change_moves_the_address() {
        // Making a thing runnable is a change to what it is, so the payload is
        // structure and stays in the skeleton.
        let mut a = valve(["halt", "stop"]);
        a["payload"] = json!({"kind": "none"});
        let mut b = a.clone();
        b["payload"] = json!({"kind": "wasm", "module": "x.wasm"});
        assert_ne!(
            DuUuid::generate(&a, None).unwrap(),
            DuUuid::generate(&b, None).unwrap()
        );
    }

    #[test]
    fn a_state_space_change_moves_the_address() {
        let mut a = valve(["halt", "stop"]);
        a["state_space"] = json!({"pressure": {"type": "float"}});
        let mut b = a.clone();
        b["state_space"] = json!({"flow_rate": {"type": "float"}});
        assert_ne!(
            DuUuid::generate(&a, None).unwrap(),
            DuUuid::generate(&b, None).unwrap()
        );
    }
}

#[cfg(test)]
mod capability_tests {
    //! The portable half of a manifest: what a thing can do, with no reference to
    //! where it is.
    //!
    //! The split between *capability* and *binding* already exists in the
    //! addressing and did not need building: `skeleton` drops `resource_id` and
    //! `external_id` as surface, so the content address of a manifest is an
    //! address of what it can do. That is exactly the property a system which
    //! attaches arbitrary new backends needs, and these tests are what say so
    //! rather than leave it as a coincidence of a deny-list.
    //!
    //! It is also the answer to "does the artifact or the source travel". The
    //! capability address is the same on every architecture; the full manifest
    //! address is not, because a GPU instance and a CPU instance of one capability
    //! are different resources occupying different addresses.

    use super::*;
    use crate::bridge::primitive::{StateType, UreAction, UreResource};
    use std::collections::HashMap;

    fn ure(resource_id: &str) -> UreResource {
        UreResource {
            ure_version: "1.0".to_string(),
            resource_id: resource_id.to_string(),
            category: "actuator".to_string(),
            state_space: HashMap::new(),
            action_primitives: vec![UreAction {
                id: "set_value".to_string(),
                aliases: None,
                params: HashMap::new(),
                target_state: "x = 1".to_string(),
                constraints: vec![],
            }],
        }
    }

    fn address_of(manifest: &serde_json::Value) -> String {
        DuUuid::generate(&skeleton(manifest), None)
            .map(|u| u.to_string())
            .unwrap_or_else(|e| format!("unaddressable: {e}"))
    }

    /// **The portable half travels and the whole does not.** Two resources that
    /// declare the same acts and sit at different ids share a capability address
    /// and differ as artifacts. This is the property that lets a linker bind a
    /// demand to "something that can set a value" without knowing or caring which
    /// backend is behind it, and it is what makes a `.ure` reusable across
    /// x86-64, wasm32 and a GPU without a rewrite.
    #[test]
    fn a_capability_address_is_the_same_wherever_the_resource_is() {
        let on_cpu = ure("cpu-0");
        let on_gpu = ure("gpu-0");

        let cap_cpu = address_of(&serde_json::to_value(&on_cpu).expect("serialises"));
        let cap_gpu = address_of(&serde_json::to_value(&on_gpu).expect("serialises"));
        assert_eq!(
            cap_cpu, cap_gpu,
            "the same acts at two addresses are one capability, which is the whole \\
             reason a demand can bind without naming a machine"
        );

        // **And there is no second address to differ.** I wrote this assertion
        // expecting a "full artifact" hash that included `resource_id`, on the
        // theory that the capability was the portable half of a split. There
        // isn't one: `DuUuid::generate` reduces to `skeleton` *first*, so every
        // address this project computes is a capability address and `resource_id`
        // cannot reach any of them. The design is stronger than the plan I wrote
        // for it.
        //
        // The consequence is the useful one: **a resource cannot contain its own
        // identity in its own identity computation.** That is why `induce` can
        // stamp a manifest with its own address, and it is the same guarantee
        // that makes an address portable to a backend that has never seen the
        // resource. The binding lives outside the address entirely -- in
        // `resource_id`, which is not hashed, and in `Linker::routes`, which is a
        // separate table keyed by `capability_key`. So the split I said the format
        // needed is already in the right place and the format is not the problem.
        let plain = DuUuid::generate(&serde_json::to_value(&on_cpu).expect("s"), None)
            .map(|u| u.to_string())
            .expect("hashes");
        assert_eq!(
            plain, cap_cpu,
            "the plain address and the skeleton address are the same value, \
             because generate() reduces to the skeleton before hashing. There is \
             no address anywhere in this project that includes where a resource \
             lives, and a manifest can therefore be stamped with its own."
        );
    }

    /// **A different capability is a different address.** Without this the test
    /// above would pass for a hash that ignored the manifest entirely, which is
    /// the failure a content address must never have.
    #[test]
    fn a_different_capability_is_a_different_address() {
        let mut other = ure("cpu-0");
        other.action_primitives[0].id = "toggle".to_string();
        assert_ne!(
            address_of(&serde_json::to_value(&ure("cpu-0")).expect("s")),
            address_of(&serde_json::to_value(&other).expect("s")),
            "two different acts are two capabilities, so the address is not a \\
             constant"
        );
    }

    /// **Surface changes do not move the capability.** A rename, a description,
    /// a guidance string: the creature is the same capability wearing different
    /// words. This is the surface/structure split the deny-list exists for, and
    /// it is the reason a peer can re-label a resource without invalidating
    /// every binding that points at it.
    #[test]
    fn surface_does_not_move_the_capability_address() {
        let bare = ure("cpu-0");
        let mut dressed = bare.clone();
        dressed.action_primitives[0].aliases = Some(vec!["set".into(), "write it".into()]);
        dressed.action_primitives[0].constraints = vec!["status != 'fault'".into()];

        assert_eq!(
            address_of(&serde_json::to_value(&bare).expect("s")),
            address_of(&serde_json::to_value(&dressed).expect("s")),
            "aliases and preconditions are surface: they change how a capability is \
             reached for and spoken to, not what it can do. Preconditions are the \
             harder call and the deny-list is the reason they are treated as \
             surface -- a gate that moved the address would make every binding to \
             a guarded capability a dangling reference."
        );
    }
}
